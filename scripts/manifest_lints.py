"""Translate authenticated package lint tables to ordered host compiler policy."""
import re

MAX_LINTS = 4096
MAX_BYTES = 256 * 1024
LEVELS = frozenset(('allow', 'warn', 'deny', 'forbid'))


def cap_arguments(level):
    if level is None:
        return ()
    if not isinstance(level, str) or level not in LEVELS:
        raise ValueError('invalid explicit host lint cap')
    return ('--cap-lints', level)


def arguments(table):
    if not isinstance(table, dict) or set(table) - {'rust', 'clippy'}:
        raise ValueError('host manifest lint tool or workspace inheritance is not implemented')
    rows, checks = [], []
    for tool, lints in table.items():
        if not isinstance(lints, dict):
            raise ValueError('host manifest lints require tool tables')
        for name, value in lints.items():
            if not isinstance(name, str) or not re.fullmatch(r'[A-Za-z_][A-Za-z_0-9-]*', name):
                raise ValueError('invalid host manifest lint name')
            if isinstance(value, str):
                value = {'level': value}
            if not isinstance(value, dict) or set(value) - {'level', 'priority', 'check-cfg'}:
                raise ValueError('unsupported host manifest lint setting')
            level, priority = value.get('level'), value.get('priority', 0)
            if not isinstance(level, str) or level not in LEVELS:
                raise ValueError('invalid host manifest lint level')
            if type(priority) is not int or not -128 <= priority <= 127:
                raise ValueError('host manifest lint priority must fit i8')
            lint = name if tool == 'rust' else f'{tool}::{name}'
            rows.append((priority, lint, level))
            if len(rows) > MAX_LINTS:
                raise ValueError('host manifest lints exceed count budget')
            if 'check-cfg' in value:
                values = value['check-cfg']
                if (tool != 'rust' or name != 'unexpected_cfgs' or not isinstance(values, list)
                        or len(values) > MAX_LINTS or any(not isinstance(item, str)
                            or not item or any(ord(ch) < 32 for ch in item) for item in values)):
                    raise ValueError('unsupported or malformed manifest check-cfg setting')
                checks.extend(values)
                if len(checks) > MAX_LINTS:
                    raise ValueError('host manifest check-cfg exceeds count budget')
    # Cargo 1.98 orders equal-priority qualified names in reverse lexical order.
    # Priority remains the outer ordering, so explicit overrides follow groups.
    rows.sort(key=lambda row: row[1], reverse=True)
    rows.sort(key=lambda row: row[0])
    result = [f'--{level}={name}' for _, name, level in rows]
    for check in checks:
        result.extend(('--check-cfg', check))
    if sum(len(value.encode()) for value in result) > MAX_BYTES:
        raise ValueError('host manifest lint arguments exceed byte budget')
    return tuple(result)
