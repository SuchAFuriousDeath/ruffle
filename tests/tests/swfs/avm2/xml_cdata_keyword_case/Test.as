package {
    import flash.display.Sprite;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        public function Test() {
            var cases:Array = [
                "<text><![CDATA[ a <> b ]]></text>",
                "<text><![cdata[ a <> b ]]></text>",
                "<text><![Cdata[ a <> b ]]></text>",
                "<text><![cDaTa[ a <> b ]]></text>",
                "<text>x<![cdata[y]]>z</text>",
                "<text><![cdata[ unterminated </text>",
                "<text><![cdatax[ y ]]></text>",
                "<text><![cdat[ y ]]></text>",
                "<text><![ cdata[ y ]]></text>"
            ];
            for each (var s:String in cases) {
                try {
                    var xml:XML = new XML(s);
                    trace(s + " => " + xml.toXMLString().replace(/\n\s*/g, " | ") + " children=" + xml.children().length());
                } catch (e:Error) {
                    trace(s + " => " + getQualifiedClassName(e) + " #" + e.errorID);
                }
            }
            for each (var t:String in ["<![cdata[top]]>", "<![CDATA[top]]>"]) {
                try {
                    var x:XML = new XML(t);
                    trace(t + " => kind=" + x.nodeKind() + " " + x.toXMLString());
                } catch (e:Error) {
                    trace(t + " => " + getQualifiedClassName(e) + " #" + e.errorID);
                }
            }
        }
    }
}
