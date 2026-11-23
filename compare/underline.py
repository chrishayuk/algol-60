"""Write the word delimiters of an ALGOL 60 program underlined (U+0332),
the representation read by both algol60 and sergev's x1sim."""
import re, sys
WORDS = ("begin end comment real integer Boolean array procedure switch label "
         "string value own if then else for do step until while goto true false").split()
def underline(text):
    def u(m): return ''.join(c + '̲' for c in m.group(0))
    out, i = [], 0
    for m in re.finditer(r"`[^']*'", text):   # leave strings alone
        out.append(re.sub(r'\b(' + '|'.join(WORDS) + r')\b', u, text[i:m.start()]))
        out.append(m.group(0)); i = m.end()
    out.append(re.sub(r'\b(' + '|'.join(WORDS) + r')\b', u, text[i:]))
    return ''.join(out)
if __name__ == '__main__':
    sys.stdout.write(underline(sys.stdin.read()))
