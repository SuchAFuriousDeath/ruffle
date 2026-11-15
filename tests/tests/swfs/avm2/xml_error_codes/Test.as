package {
    import flash.display.Sprite;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        public function Test() {
            var cases:Array = [
                ["whitespace before double quote", '<a b= "x'],
                ["whitespace before single quote", "<a b=  'x"],
                ["unterminated value hits <", '<a b="x<c/>'],
                ["unterminated value hits < after =", '<a b="x" c=\'y<d/>'],
                ["< ends unterminated value before a later quote", '<a b="x<c d="y"'],
                ["< ends unterminated single-quoted value before a later quote", "<a b='x<c d='y'"],
                ["< in closed value", '<a b="x<y"/>'],
                ["< in closed single-quoted value", "<a b='<'/>"],
                ["quote not after =", '<a b"x"/>'],
                ["unquoted value", '<a b=x'],
                ["unterminated tag", '<a b="x" c'],
                ["<?xml without space", '<?xml'],
                ["<?xmlfoo", '<?xmlfoo bar'],
                ["<?xml with space", '<?xml version="1.0"'],
                ["<?XML uppercase", '<?XML version="1.0"'],
                ["<?xml with tab", '<?xml\tversion="1.0"'],
                ["other PI", '<?foo bar']
            ];
            for each (var c:Array in cases) {
                try {
                    new XML(c[1]);
                    trace(c[0] + ": no error");
                } catch (e:Error) {
                    trace(c[0] + ": " + getQualifiedClassName(e) + " #" + e.errorID);
                }
            }
        }
    }
}
