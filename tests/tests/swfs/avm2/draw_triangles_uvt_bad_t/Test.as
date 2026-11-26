package {
import flash.display.BitmapData;
import flash.display.Shape;
import flash.display.Sprite;

// Checks how triangles with zero, negative or non-finite t values and
// non-finite u/v values are drawn. In each cell, the upper-left triangle
// gets the tested values and the lower-right triangle is a normal one.
[SWF(width="400", height="300", backgroundColor="#FFFFFF")]
public class Test extends Sprite {
    private var cellId:int = 0;
    private var bmd:BitmapData;

    public function Test() {
        bmd = new BitmapData(4, 4, false, 0);
        for (var y:int = 0; y < 4; y++) {
            for (var x:int = 0; x < 4; x++) {
                bmd.setPixel(x, y, ((x * 80) << 16) | ((y * 80) << 8) | ((x + y) % 2 == 0 ? 0xFF : 0x00));
            }
        }

        // Upper-left triangle: vertices 0, 1, 2; lower-right: 3, 4, 5.
        draw("t all 0", [0, 0, 0], [1, 0, 0], [0, 1, 0]);
        draw("t all -1", [0, 0, -1], [1, 0, -1], [0, 1, -1]);
        draw("t one 0", [0, 0, 0], [1, 0, 1], [0, 1, 1]);
        draw("t one -1", [0, 0, -1], [1, 0, 1], [0, 1, 1]);
        draw("t one NaN", [0, 0, NaN], [1, 0, 1], [0, 1, 1]);
        draw("t one Infinity", [0, 0, Infinity], [1, 0, 1], [0, 1, 1]);
        draw("t tiny", [0, 0, 1e-30], [1, 0, 1e-30], [0, 1, 1e-30]);
        draw("t one tiny", [0, 0, 1e-30], [1, 0, 1], [0, 1, 1]);
        draw("u one NaN", [NaN, 0, 1], [1, 0, 1], [0, 1, 1]);
        draw("v one Infinity", [0, Infinity, 1], [1, 0, 1], [0, 1, 1]);
        draw("t huge", [0, 0, 1e30], [1, 0, 1], [0, 1, 1]);
        draw("t all -1 mixed", [0, 0, -1], [1, 0, -0.5], [0, 1, -2]);

        trace("Done");
    }

    private function draw(name:String, a:Array, b:Array, c:Array):void {
        var s:Shape = new Shape();
        s.x = (cellId % 4) * 100 + 10;
        s.y = Math.floor(cellId / 4) * 100 + 10;
        addChild(s);

        var uvt:Vector.<Number> = new Vector.<Number>();
        uvt.push(a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]);
        uvt.push(1, 0, 1, 1, 1, 1, 0, 1, 1);

        s.graphics.beginBitmapFill(bmd);
        try {
            s.graphics.drawTriangles(Vector.<Number>([0, 0, 76, 0, 0, 76, 80, 4, 80, 80, 4, 80]), null, uvt);
            trace(cellId + " " + name + ": rect " + s.getRect(s));
        } catch (e:Error) {
            trace(cellId + " " + name + ": " + e);
        }
        s.graphics.endFill();
        ++cellId;
    }
}
}
