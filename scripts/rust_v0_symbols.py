"""Structured parser for Rust v0 mangled symbols (RFC 2603 plus const/pattern extensions).

Ported from the grammar implemented by rustc-demangle 0.1.27 (`src/v0.rs`), the
copy vendored with the pinned 1.98.0 `rust-src`. Unlike a plain demangler this
keeps the impl path that `M`/`X` nodes carry, which names the crate and module
that *define* a method even when its self type belongs to another crate, and it
can erase generic arguments to name the source definition behind an instance.
"""
from functools import lru_cache

BASIC = {
    'a': 'i8', 'b': 'bool', 'c': 'char', 'd': 'f64', 'e': 'str', 'f': 'f32', 'h': 'u8',
    'i': 'isize', 'j': 'usize', 'l': 'i32', 'm': 'u32', 'n': 'i128', 'o': 'u128',
    's': 'i16', 't': 'u16', 'u': '()', 'v': '...', 'x': 'i64', 'y': 'u64', 'z': '!', 'p': '_',
}
DIGITS62 = {c: i for i, c in enumerate(
    '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ')}


class Invalid(ValueError):
    """The text is not a v0 symbol this grammar accepts."""


class Parser:
    """Recursive-descent parser producing nested tuples; backrefs are memoized."""

    def __init__(self, sym):
        self.sym, self.next, self.memo = sym, 0, {}

    def peek(self):
        return self.sym[self.next] if self.next < len(self.sym) else ''

    def take(self):
        if self.next >= len(self.sym):
            raise Invalid('truncated')
        char = self.sym[self.next]
        self.next += 1
        return char

    def eat(self, char):
        if self.peek() == char:
            self.next += 1
            return True
        return False

    def integer_62(self):
        if self.eat('_'):
            return 0
        value = 0
        while not self.eat('_'):
            digit = DIGITS62.get(self.take())
            if digit is None:
                raise Invalid('base-62 digit')
            value = value * 62 + digit
        return value + 1

    def opt_integer_62(self, tag):
        return self.integer_62() + 1 if self.eat(tag) else 0

    def ident(self):
        punycode = self.eat('u')
        char = self.take()
        if not char.isdigit():
            raise Invalid('identifier length')
        length = int(char)
        if length:
            while self.peek().isdigit():
                length = length * 10 + int(self.take())
        self.eat('_')
        text = self.sym[self.next:self.next + length]
        if len(text) != length:
            raise Invalid('identifier bytes')
        self.next += length
        if not punycode:
            return text
        head, _, tail = text.rpartition('_')
        if not tail:
            raise Invalid('punycode identifier')
        try:
            return (head + '-' + tail).encode('ascii').decode('punycode')
        except (UnicodeError, ValueError) as error:
            raise Invalid('punycode identifier') from error

    def backref(self, parse):
        start = self.next - 1
        target = self.integer_62()
        if target >= start:
            raise Invalid('forward backref')
        key = (target, parse.__name__)
        if key not in self.memo:
            saved, self.next = self.next, target
            try:
                self.memo[key] = parse()
            finally:
                self.next = saved
        return self.memo[key]

    def path(self):
        tag = self.take()
        if tag == 'C':
            dis = self.opt_integer_62('s')
            return ('crate', self.ident(), dis)
        if tag == 'N':
            namespace = self.take()
            if not namespace.isalpha():
                raise Invalid('namespace')
            inner = self.path()
            dis = self.opt_integer_62('s')
            return ('nested', namespace, inner, self.ident(), dis)
        if tag in 'MX':
            dis = self.opt_integer_62('s')
            impl_path = self.path()
            self_type = self.type()
            trait = self.path() if tag == 'X' else None
            return ('impl', impl_path, dis, self_type, trait)
        if tag == 'Y':
            self_type = self.type()
            return ('trait', self_type, self.path())
        if tag == 'I':
            inner = self.path()
            return ('generic', inner, self.list(self.generic_arg))
        if tag == 'B':
            return self.backref(self.path)
        raise Invalid('path tag ' + tag)

    def list(self, parse):
        items = []
        while not self.eat('E'):
            items.append(parse())
        return tuple(items)

    def generic_arg(self):
        if self.eat('L'):
            return ('lifetime', self.integer_62())
        if self.eat('K'):
            return ('const', self.const())
        return self.type()

    def binder(self):
        return self.opt_integer_62('G')

    def type(self):
        tag = self.take()
        if tag in BASIC:
            return ('basic', BASIC[tag])
        if tag in 'RQ':
            lifetime = self.integer_62() if self.eat('L') else 0
            return ('ref', lifetime, tag == 'Q', self.type())
        if tag in 'PO':
            return ('ptr', tag == 'O', self.type())
        if tag == 'A':
            element = self.type()
            return ('array', element, self.const())
        if tag == 'S':
            return ('slice', self.type())
        if tag == 'T':
            return ('tuple', self.list(self.type))
        if tag == 'F':
            bound = self.binder()
            unsafe = self.eat('U')
            abi = None
            if self.eat('K'):
                abi = 'C' if self.eat('C') else self.ident().replace('_', '-')
            parameters = self.list(self.type)
            return ('fn', bound, unsafe, abi, parameters, self.type())
        if tag == 'D':
            bound = self.binder()
            traits = self.list(self.dyn_trait)
            if not self.eat('L'):
                raise Invalid('dyn lifetime')
            return ('dyn', bound, traits, self.integer_62())
        if tag == 'B':
            return self.backref(self.type)
        if tag == 'W':
            base = self.type()
            return ('pattern', base, self.pattern())
        self.next -= 1
        return ('named', self.path())

    def dyn_trait(self):
        trait = self.path()
        bindings = []
        while self.eat('p'):
            name = self.ident()
            bindings.append((name, ('const', self.const()) if self.eat('K') else self.type()))
        return ('dyn-trait', trait, tuple(bindings))

    def pattern(self):
        tag = self.take()
        if tag == 'R':
            start = self.const()
            return ('range', start, self.const())
        if tag == 'O':
            return ('or', self.list(self.pattern))
        if tag == 'N':
            return ('not-null',)
        raise Invalid('pattern tag ' + tag)

    def hex_nibbles(self):
        start = self.next
        while self.peek() and self.peek() in '0123456789abcdef':
            self.next += 1
        text = self.sym[start:self.next]
        if not self.eat('_'):
            raise Invalid('hex terminator')
        return text

    def const(self):
        tag = self.take()
        if tag == 'p':
            return '_'
        if tag in 'htmyoj':
            return self.const_uint()
        if tag in 'aslxni':
            sign = '-' if self.eat('n') else ''
            return sign + self.const_uint()
        if tag == 'b':
            return {'0': 'false', '1': 'true'}.get(self.hex_nibbles().lstrip('0') or '0', '?')
        if tag == 'c':
            return repr(chr(int(self.hex_nibbles() or '0', 16)))
        if tag == 'e':
            return '*' + self.const_str()
        if tag in 'RQ':
            if tag == 'R' and self.eat('e'):
                return self.const_str()
            return ('&mut ' if tag == 'Q' else '&') + self.const()
        if tag == 'A':
            return '[' + ', '.join(self.list(self.const)) + ']'
        if tag == 'T':
            items = self.list(self.const)
            return '(' + ', '.join(items) + (',' if len(items) == 1 else '') + ')'
        if tag == 'V':
            return self.const_variant()
        if tag == 'B':
            return self.backref(self.const)
        raise Invalid('const tag ' + tag)

    def const_uint(self):
        nibbles = self.hex_nibbles()
        value = int(nibbles or '0', 16)
        return str(value) if value < 1 << 64 else '0x' + nibbles

    def const_str(self):
        nibbles = self.hex_nibbles()
        try:
            return '"' + bytes.fromhex(nibbles).decode('utf-8') + '"'
        except ValueError as error:
            raise Invalid('string constant') from error

    def const_variant(self):
        name = render_path(self.path(), True, [0])
        shape = self.take()
        if shape == 'U':
            return name
        if shape == 'T':
            return name + '(' + ', '.join(self.list(self.const)) + ')'
        if shape == 'S':
            fields = []
            while not self.eat('E'):
                self.opt_integer_62('s')
                field = self.ident()
                fields.append(field + ': ' + self.const())
            return name + ' { ' + ', '.join(fields) + ' }'
        raise Invalid('constant variant shape')


def lifetime_name(index, depth):
    """The letter rustc-demangle gives a bound lifetime at the current binder depth."""
    if index == 0:
        return "'_"
    level = depth[0] - index
    if level < 0:
        raise Invalid('unbound lifetime')
    return "'" + (chr(ord('a') + level) if level < 26 else '_' + str(level))


def open_binder(count, depth):
    if not count:
        return ''
    names = []
    for _ in range(count):
        depth[0] += 1
        names.append(lifetime_name(1, depth))
    return 'for<' + ', '.join(names) + '> '


DEMANGLE_LABELS = {'C': 'closure', 'S': 'shim'}
# rustc's definition-path spelling of the special namespaces the v0 mangler emits.
DEFINITION_LABELS = {'C': 'closure', 'S': 'shim', 'K': 'constant'}


def segment(namespace, name, dis, labels=DEMANGLE_LABELS):
    if namespace.isupper():
        label = labels.get(namespace, namespace)
        return '::{' + label + (':' + name if name else '') + '#' + str(dis) + '}'
    return '::' + name if name else ''


def render_arg(node, depth):
    if node[0] == 'lifetime':
        return lifetime_name(node[1], depth)
    if node[0] == 'const':
        return node[1]
    return render_type(node, depth)


def render_path(node, in_value, depth):
    """Text in rustc-demangle's alternate (`{:#}`) form: no crate hashes."""
    kind = node[0]
    if kind == 'crate':
        return node[1]
    if kind == 'nested':
        return render_path(node[2], in_value, depth) + segment(node[1], node[3], node[4])
    if kind == 'impl':
        text = '<' + render_type(node[3], depth)
        if node[4] is not None:
            text += ' as ' + render_path(node[4], False, depth)
        return text + '>'
    if kind == 'trait':
        return '<' + render_type(node[1], depth) + ' as ' + render_path(node[2], False, depth) + '>'
    arguments = ', '.join(render_arg(argument, depth) for argument in node[2])
    return render_path(node[1], in_value, depth) + ('::<' if in_value else '<') + arguments + '>'


def render_dyn_trait(node, depth):
    trait, bindings = node[1], node[2]
    if trait[0] == 'generic':
        parts = [render_arg(argument, depth) for argument in trait[2]]
        text = render_path(trait[1], False, depth)
    else:
        parts, text = [], render_path(trait, False, depth)
    parts += [name + ' = ' + render_arg(value, depth) for name, value in bindings]
    return text + ('<' + ', '.join(parts) + '>' if parts else '')


def render_pattern(node):
    if node[0] == 'range':
        return node[1] + '..=' + node[2]
    if node[0] == 'or':
        return ' | '.join(render_pattern(item) for item in node[1])
    return '!null'


def render_type(node, depth):
    kind = node[0]
    if kind == 'basic':
        return node[1]
    if kind == 'named':
        return render_path(node[1], False, depth)
    if kind == 'ref':
        lifetime = lifetime_name(node[1], depth) + ' ' if node[1] else ''
        return '&' + lifetime + ('mut ' if node[2] else '') + render_type(node[3], depth)
    if kind == 'ptr':
        return ('*mut ' if node[1] else '*const ') + render_type(node[2], depth)
    if kind == 'array':
        return '[' + render_type(node[1], depth) + '; ' + node[2] + ']'
    if kind == 'slice':
        return '[' + render_type(node[1], depth) + ']'
    if kind == 'tuple':
        items = [render_type(item, depth) for item in node[1]]
        return '(' + ', '.join(items) + (',' if len(items) == 1 else '') + ')'
    if kind == 'fn':
        saved = depth[0]
        text = open_binder(node[1], depth) + ('unsafe ' if node[2] else '')
        if node[3] is not None:
            text += 'extern "' + node[3] + '" '
        text += 'fn(' + ', '.join(render_type(item, depth) for item in node[4]) + ')'
        if node[5] != ('basic', '()'):
            text += ' -> ' + render_type(node[5], depth)
        depth[0] = saved
        return text
    if kind == 'dyn':
        saved = depth[0]
        text = 'dyn ' + open_binder(node[1], depth)
        text += ' + '.join(render_dyn_trait(item, depth) for item in node[2])
        depth[0] = saved
        return text + (' + ' + lifetime_name(node[3], depth) if node[3] else '')
    if kind == 'pattern':
        return render_type(node[1], depth) + ' is ' + render_pattern(node[2])
    raise Invalid('type node ' + kind)


def defining_path(node):
    """The definition behind an instance: generic arguments erased, impls by index.

    Impl blocks are named `{impl#N}` using the disambiguator rustc assigns within
    the parent module, the same number rustc prints in a definition path.
    """
    kind = node[0]
    if kind == 'crate':
        return node[1]
    if kind == 'nested':
        return defining_path(node[2]) + segment(node[1], node[3], node[4], DEFINITION_LABELS)
    if kind == 'impl':
        return defining_path(node[1]) + '::{impl#' + str(node[2]) + '}'
    if kind == 'trait':
        return defining_path(node[2])
    return defining_path(node[1])


PARENT = {'nested': 2, 'impl': 1, 'trait': 2, 'generic': 1}


def defining_crate(node):
    """Crate holding the definition: the impl's crate, or a default method's trait crate."""
    while node[0] != 'crate':
        node = node[PARENT[node[0]]]
    return node[1]


def module_path(node):
    """Leading type-namespace segments: the module (or trait) holding the definition."""
    spine = []
    while node[0] != 'crate':
        spine.append(node)
        node = node[PARENT[node[0]]]
    parts = [node[1]]
    for item in reversed(spine):
        if item[0] == 'generic':
            continue
        if item[0] != 'nested' or item[1] != 't':
            break
        parts.append(item[3])
    return '::'.join(parts)


def classify(node):
    """`closure`, `shim`, `drop-glue` or `fn`, from the outermost path segments.

    In 1.98 `core::ptr::drop_in_place` is an ordinary function that calls the
    `drop_glue` lang item; the compiler replaces each `drop_glue::<T>` body.
    """
    generic = node[0] == 'generic'
    inner = node[1] if generic else node
    if inner[0] == 'nested' and inner[1] == 'C':
        return 'closure'
    if inner[0] == 'nested' and inner[1] == 'S':
        return 'shim'
    if generic and defining_path(inner) == 'core::ptr::drop_glue':
        return 'drop-glue'
    return 'fn'


def has_generic_arguments(node):
    kind = node[0]
    if kind == 'crate':
        return False
    if kind == 'generic':
        return True
    if kind == 'nested':
        return has_generic_arguments(node[2])
    return False


@lru_cache(maxsize=None)
def parse(symbol):
    """Parse `_R...`; returns (path, instantiating crate or None, suffix)."""
    if not symbol.startswith('_R') or len(symbol) < 3 or not symbol[2].isupper():
        raise Invalid('not a v0 symbol')
    if not symbol.isascii():
        raise Invalid('non-ASCII symbol')
    parser = Parser(symbol[2:])
    path = parser.path()
    instantiating = None
    if parser.peek().isupper():
        instantiating = parser.path()
    return path, instantiating, parser.sym[parser.next:]


def demangle(symbol):
    """Alternate-form text of a v0 symbol, like `rustc-demangle`'s `{:#}`."""
    path, _, _ = parse(symbol)
    return render_path(path, True, [0])
