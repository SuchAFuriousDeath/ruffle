package {
import flash.display.BitmapData;
import flash.display.GraphicsBitmapFill;
import flash.display.GraphicsEndFill;
import flash.display.GraphicsTrianglePath;
import flash.display.IGraphicsData;
import flash.display.Shape;
import flash.display.Sprite;

// Checks which uvtData lengths drawTriangles and GraphicsTrianglePath
// accept, which errors they throw, and how the drawn bounds change.
public class Test extends Sprite {
    private var bmd:BitmapData = new BitmapData(4, 4, false, 0xFF0000);

    public function Test() {
        trace("===== drawTriangles");
        run(function(g:*, v:Vector.<Number>, i:Vector.<int>, uvt:Vector.<Number>, c:String):void {
            g.drawTriangles(v, i, uvt, c);
        });

        trace("===== GraphicsTrianglePath");
        run(function(g:*, v:Vector.<Number>, i:Vector.<int>, uvt:Vector.<Number>, c:String):void {
            g.drawGraphicsData(Vector.<IGraphicsData>([
                new GraphicsBitmapFill(bmd),
                new GraphicsTrianglePath(v, i, uvt, c),
                new GraphicsEndFill()
            ]));
        });

        trace("Done");
    }

    private function nums(n:int):Vector.<Number> {
        var r:Vector.<Number> = new Vector.<Number>();
        for (var k:int = 0; k < n; k++) {
            r.push(k % 2 == 0 ? 0.5 : 0.25);
        }
        return r;
    }

    private function attempt(name:String, draw:Function, v:Vector.<Number>, i:Vector.<int>, uvt:Vector.<Number>, c:String = "none"):void {
        var s:Shape = new Shape();
        s.graphics.beginBitmapFill(bmd);
        try {
            draw(s.graphics, v, i, uvt, c);
            s.graphics.endFill();
            trace(name + ": ok, rect " + s.getRect(s));
        } catch (e:Error) {
            trace(name + ": " + Object(e).constructor + " " + e.errorID + " " + e.message + ", rect " + s.getRect(s));
        }
    }

    private function run(draw:Function):void {
        // 3 vertices, sequential.
        var v3:Vector.<Number> = Vector.<Number>([0, 0, 10, 0, 0, 10]);
        for (var n:int = 0; n <= 10; n++) {
            attempt("3 vertices, null indices, uvt length " + n, draw, v3, null, nums(n));
        }

        // 4 vertices, indexed, 2 triangles.
        var v4:Vector.<Number> = Vector.<Number>([0, 0, 10, 0, 0, 10, 10, 10]);
        var i6:Vector.<int> = Vector.<int>([0, 1, 2, 1, 2, 3]);
        for (n = 0; n <= 14; n++) {
            attempt("4 vertices, 6 indices, uvt length " + n, draw, v4, i6, nums(n));
        }

        // 6 vertices: 12 = 2 * 6 and 12 = 3 * 4.
        var v6:Vector.<Number> = Vector.<Number>([0, 0, 10, 0, 0, 10, 20, 0, 30, 0, 20, 10]);
        attempt("6 vertices, null indices, uvt length 12", draw, v6, null, nums(12));
        attempt("6 vertices, null indices, uvt length 18", draw, v6, null, nums(18));
        attempt("6 vertices, 3 indices, uvt length 6", draw, v6, Vector.<int>([0, 1, 2]), nums(6));
        attempt("6 vertices, 3 indices, uvt length 9", draw, v6, Vector.<int>([0, 1, 2]), nums(9));

        // Index out of range with uv data.
        attempt("index out of range first, uv", draw, v4, Vector.<int>([0, 1, 7, 1, 2, 3]), nums(8));
        attempt("index out of range second, uv", draw, v4, Vector.<int>([1, 2, 3, 0, 1, 7]), nums(8));
        attempt("negative index, uv", draw, v4, Vector.<int>([1, 2, 3, -1, 1, 2]), nums(8));
        attempt("index out of range, no uv", draw, v4, Vector.<int>([1, 2, 3, 0, 1, 4]), null);

        // Odd vertices length / indices length with uv.
        attempt("5 vertex numbers, uv 4", draw, Vector.<Number>([0, 0, 10, 0, 0]), null, nums(4));
        attempt("4 vertices, 4 indices, uv 8", draw, v4, Vector.<int>([0, 1, 2, 3]), nums(8));

        // Empty inputs with uv data.
        attempt("0 vertices, uv 0", draw, new Vector.<Number>(), null, nums(0));
        attempt("0 vertices, uv 2", draw, new Vector.<Number>(), null, nums(2));
        attempt("4 vertices, 0 indices, uv 3", draw, v4, new Vector.<int>(), nums(3));

        // Culling string checked before or after uv length?
        attempt("bad culling, bad uv", draw, v3, null, nums(5), "bogus");
    }
}
}
