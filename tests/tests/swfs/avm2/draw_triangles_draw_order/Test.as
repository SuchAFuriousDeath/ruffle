package {
import flash.display.BitmapData;
import flash.display.Shape;
import flash.display.Sprite;
import flash.geom.Matrix;

// Checks how triangles from drawTriangles combine with each other and with
// an open fill path: overlaps, winding, draw order and the pen position.
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
        var big:Matrix = new Matrix();
        big.scale(20, 20);
        var g:*;

        // 0. Overlapping triangles, same winding, one call.
        g = cell("overlap same winding");
        g.beginFill(0xFF0000, 0.5);
        g.drawTriangles(Vector.<Number>([0, 0, 60, 0, 0, 60, 20, 20, 80, 20, 20, 80]));
        g.endFill();
        report();

        // 1. Overlapping triangles, opposite winding, one call.
        g = cell("overlap opposite winding");
        g.beginFill(0xFF0000, 0.5);
        g.drawTriangles(Vector.<Number>([0, 0, 60, 0, 0, 60, 20, 20, 20, 80, 80, 20]));
        g.endFill();
        report();

        // 2. Overlapping triangles, two calls in the same fill.
        g = cell("overlap two calls");
        g.beginFill(0xFF0000, 0.5);
        g.drawTriangles(Vector.<Number>([0, 0, 60, 0, 0, 60]));
        g.drawTriangles(Vector.<Number>([20, 20, 80, 20, 20, 80]));
        g.endFill();
        report();

        // 3. Overlapping bitmap uv triangles with a semi-transparent bitmap.
        g = cell("overlap uv alpha");
        var abmd:BitmapData = new BitmapData(2, 2, true, 0x80FF0000);
        g.beginBitmapFill(abmd);
        g.drawTriangles(Vector.<Number>([0, 0, 60, 0, 0, 60, 20, 20, 80, 20, 20, 80]), null,
            Vector.<Number>([0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1]));
        g.endFill();
        report();

        // 4. Open path, drawTriangles in the middle, path continues.
        g = cell("open path around");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(0, 0);
        g.lineTo(80, 0);
        g.drawTriangles(Vector.<Number>([10, 10, 70, 10, 10, 70]));
        g.lineTo(80, 80);
        g.lineTo(0, 80);
        g.endFill();
        report();

        // 5. drawRect first in the same fill, then overlapping triangles.
        g = cell("rect then triangles same fill");
        g.beginFill(0xFF0000, 0.5);
        g.drawRect(30, 30, 50, 50);
        g.drawTriangles(Vector.<Number>([0, 0, 80, 0, 0, 80]));
        g.endFill();
        report();

        // 6. Triangles first, then drawRect in a different fill overlapping.
        g = cell("triangles then rect");
        g.beginFill(0xFF0000);
        g.drawTriangles(Vector.<Number>([0, 0, 80, 0, 0, 80]));
        g.beginFill(0x0000FF);
        g.drawRect(30, 30, 50, 50);
        g.endFill();
        report();

        // 7. Bitmap fill: open path (fill matrix) overlapping uv triangles. endFill
        // closes the path to the moveTo point and not to the pen, so its fill
        // spills to the right, which is why this is the last cell of its row.
        g = cell("bitmap path vs uv");
        g.beginBitmapFill(bmd, big);
        g.moveTo(0, 0);
        g.lineTo(80, 0);
        g.lineTo(80, 50);
        g.drawTriangles(Vector.<Number>([20, 20, 80, 20, 20, 80]), null,
            Vector.<Number>([0, 0, 1, 0, 0, 1]));
        g.lineTo(0, 50);
        g.endFill();
        report();

        // 8. Pen position after drawTriangles.
        g = cell("pen position");
        g.lineStyle(2, 0x000000);
        g.moveTo(5, 75);
        g.beginFill(0x00FF00);
        g.drawTriangles(Vector.<Number>([10, 10, 70, 10, 10, 70]));
        g.endFill();
        g.lineTo(75, 75);
        report();

        // 9. Bitmap path before uv triangles, then a solid fill path.
        g = cell("bitmap path then uv then solid");
        g.beginBitmapFill(bmd, big);
        g.drawRect(0, 0, 50, 50);
        g.drawTriangles(Vector.<Number>([20, 20, 80, 20, 20, 80]), null,
            Vector.<Number>([0, 0, 1, 0, 0, 1]));
        g.beginFill(0x00FF00, 0.5);
        g.drawRect(40, 40, 40, 40);
        g.endFill();
        report();

        // 10. Fill opened with moveTo only, then drawTriangles.
        g = cell("moveTo only then triangles");
        g.beginFill(0xFF0000);
        g.moveTo(40, 40);
        g.drawTriangles(Vector.<Number>([0, 0, 80, 0, 0, 80]));
        g.endFill();
        report();

        // 11. Three overlapping triangles (winding counts 1, 2, 3).
        g = cell("triple overlap");
        g.beginFill(0x0000FF, 0.3);
        g.drawTriangles(Vector.<Number>([0, 0, 60, 0, 0, 60, 10, 10, 70, 10, 10, 70, 20, 20, 80, 20, 20, 80]));
        g.endFill();
        report();

        trace("Done");
    }

    private var current:Shape;
    private var currentName:String;

    private function cell(name:String):* {
        current = new Shape();
        current.x = (cellId % 4) * 100 + 10;
        current.y = Math.floor(cellId / 4) * 100 + 10;
        currentName = cellId + " " + name;
        ++cellId;
        addChild(current);
        return current.graphics;
    }

    private function report():void {
        trace(currentName + ": rect " + current.getRect(current));
    }
}
}
