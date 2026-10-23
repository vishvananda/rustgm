"""Translate the supported pinned Cargo profile without dropping settings."""
import json

from build_units import fields


def decode(encoded):
    profile = json.loads(encoded)
    fields(profile, 'name opt_level lto codegen_backend codegen_units debuginfo '
           'split_debuginfo debug_assertions overflow_checks rpath incremental panic strip')
    if profile['name'] not in ('dev', 'release'):
        raise ValueError('custom host profile inheritance is not implemented')
    if profile['opt_level'] not in ('0', '1', '2', '3', 's', 'z'):
        raise ValueError('unsupported host optimization level')
    if profile['lto'] not in ('false', 'off', 'thin', 'fat', 'true'):
        raise ValueError('unsupported host LTO profile')
    if profile['codegen_backend'] is not None or profile['incremental'] is not False:
        raise ValueError('host backend overrides and incremental caches are unsupported')
    count = profile['codegen_units']
    if count is not None and (type(count) is not int or not 1 <= count <= 256):
        raise ValueError('host codegen unit count is invalid or exceeds budget')
    if type(profile['debuginfo']) is not int or profile['debuginfo'] not in (0, 1, 2):
        raise ValueError('unsupported host debug information profile')
    if profile['split_debuginfo'] not in (None, 'off', 'packed', 'unpacked'):
        raise ValueError('unsupported host split debug information profile')
    if any(type(profile[key]) is not bool for key in ('debug_assertions', 'overflow_checks', 'rpath')):
        raise ValueError('host profile switches must be booleans')
    if profile['panic'] not in ('abort', 'unwind'):
        raise ValueError('unsupported host panic profile')
    # Preserve observed wire variants, including Cargo's inferred debug=0 strip.
    if profile['strip'] not in ({'deferred': 'None'}, {'resolved': {'Named': 'debuginfo'}}):
        raise ValueError('unsupported host strip protocol')
    return profile


def arguments(encoded):
    profile = decode(encoded)
    settings = [('opt-level', profile['opt_level']), ('debuginfo', str(profile['debuginfo'])),
                ('panic', profile['panic']), ('embed-bitcode', 'no' if profile['lto'] in ('false', 'off') else 'yes')]
    for key in ('debug_assertions', 'overflow_checks', 'rpath'):
        settings.append((key.replace('_', '-'), 'yes' if profile[key] else 'no'))
    # Cargo false permits rustc's local thin LTO; -Clto=off would disable it.
    if profile['lto'] != 'false':
        settings.append(('lto', 'fat' if profile['lto'] == 'true' else profile['lto']))
    if profile['strip'] == {'resolved': {'Named': 'debuginfo'}}:
        settings.append(('strip', 'debuginfo'))
    for key in ('codegen_units', 'split_debuginfo'):
        if profile[key] is not None:
            settings.append((key.replace('_', '-'), str(profile[key])))
    return tuple(part for key, value in settings for part in ('-C', f'{key}={value}'))


def environment(encoded):
    profile = decode(encoded)
    return {'DEBUG': str(profile['debuginfo'] != 0).lower(), 'OPT_LEVEL': profile['opt_level'],
            'PROFILE': 'debug' if profile['name'] == 'dev' else 'release'}
