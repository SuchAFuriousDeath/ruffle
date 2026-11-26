package {
import flash.display.BitmapData;
import flash.display.GraphicsBitmapFill;
import flash.display.GraphicsEndFill;
import flash.display.GraphicsPath;
import flash.display.GraphicsSolidFill;
import flash.display.GraphicsStroke;
import flash.display.GraphicsTrianglePath;
import flash.display.IGraphicsData;
import flash.display.Shape;
import flash.display.Sprite;
import flash.utils.getQualifiedClassName;

// Checks what readGraphicsData returns after drawTriangles.
public class Test extends Sprite {
    public function Test() {
        var bmd:BitmapData = new BitmapData(4, 4, false, 0xFF0000);
        var s:Shape;

        trace("// solid, indexed, culling positive");
        s = new Shape();
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawTriangles(Vector.<Number>([0, 0, 10, 0, 0, 10, 10, 10]), Vector.<int>([0, 1, 2, 1, 3, 2]), null, "positive");
        s.graphics.endFill();
        dump(s);

        trace("// bitmap, sequential, uvt");
        s = new Shape();
        s.graphics.beginBitmapFill(bmd);
        s.graphics.drawTriangles(Vector.<Number>([0, 0, 10, 0, 0, 10]), null, Vector.<Number>([0, 0, 1, 1, 0, 0.5, 0, 1, 0.25]));
        s.graphics.endFill();
        dump(s);

        trace("// stroke + solid, uv, between path segments");
        s = new Shape();
        s.graphics.lineStyle(2, 0x0000FF);
        s.graphics.beginFill(0x00FF00);
        s.graphics.moveTo(0, 0);
        s.graphics.lineTo(20, 0);
        s.graphics.drawTriangles(Vector.<Number>([0, 0, 10, 0, 0, 10]), null, Vector.<Number>([0, 0, 1, 0, 0, 1]));
        s.graphics.lineTo(20, 20);
        s.graphics.endFill();
        dump(s);

        trace("// GraphicsTrianglePath via drawGraphicsData");
        s = new Shape();
        s.graphics.drawGraphicsData(Vector.<IGraphicsData>([
            new GraphicsSolidFill(0x123456),
            new GraphicsTrianglePath(Vector.<Number>([0, 0, 10, 0, 0, 10]), Vector.<int>([0, 1, 2]), Vector.<Number>([0, 0, 1, 0, 0, 1]), "negative"),
            new GraphicsEndFill()
        ]));
        dump(s);

        trace("// culled away completely");
        s = new Shape();
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawTriangles(Vector.<Number>([0, 0, 10, 0, 0, 10]), null, null, "positive");
        s.graphics.endFill();
        dump(s);

        trace("// bounds: unreferenced vertex, out-of-range index, culled");
        s = new Shape();
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawTriangles(Vector.<Number>([0, 0, 10, 0, 0, 10, 100, 100]), Vector.<int>([0, 1, 2]));
        trace("  unreferenced vertex: " + s.getRect(s));
        s = new Shape();
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawTriangles(Vector.<Number>([0, 0, 10, 0, 0, 10, 100, 100]), Vector.<int>([0, 1, 2, 0, 1, 9, 1, 2, 3]));
        trace("  triangle after out-of-range index: " + s.getRect(s));
        s = new Shape();
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawTriangles(Vector.<Number>([0, 0, 10, 0, 0, 10, 100, 100, 100, 50, 50, 100]), null, null, "negative");
        trace("  second triangle culled: " + s.getRect(s));

        trace("Done");
    }

    private function dump(s:Shape):void {
        var data:Vector.<IGraphicsData>;
        try {
            data = s.graphics.readGraphicsData();
        } catch (e:Error) {
            trace("  readGraphicsData threw " + e);
            return;
        }
        trace("  " + data.length + " entries");
        for each (var d:IGraphicsData in data) {
            var line:String = "  " + getQualifiedClassName(d);
            if (d is GraphicsTrianglePath) {
                var t:GraphicsTrianglePath = GraphicsTrianglePath(d);
                line += " vertices=[" + t.vertices + "] indices=" + (t.indices ? "[" + t.indices + "]" : "null") +
                    " uvtData=" + (t.uvtData ? "[" + t.uvtData + "]" : "null") + " culling=" + t.culling;
            } else if (d is GraphicsPath) {
                var p:GraphicsPath = GraphicsPath(d);
                line += " commands=[" + p.commands + "] data=[" + p.data + "] winding=" + p.winding;
            } else if (d is GraphicsSolidFill) {
                line += " color=" + GraphicsSolidFill(d).color.toString(16) + " alpha=" + GraphicsSolidFill(d).alpha;
            } else if (d is GraphicsBitmapFill) {
                var b:GraphicsBitmapFill = GraphicsBitmapFill(d);
                line += " matrix=" + b.matrix + " repeat=" + b.repeat + " smooth=" + b.smooth;
            } else if (d is GraphicsStroke) {
                line += " thickness=" + GraphicsStroke(d).thickness + " fill=" + getQualifiedClassName(GraphicsStroke(d).fill);
            }
            trace(line);
        }
    }
}
}
