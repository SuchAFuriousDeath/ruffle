package {
import flash.display.BitmapData;
import flash.display.Shape;
import flash.display.Sprite;
import flash.geom.Matrix;

// Checks what uv/uvt data does with non-bitmap fills (solid, gradients)
// and how bitmap fills without uv data are mapped.
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

        var quad:Vector.<Number> = Vector.<Number>([0, 0, 80, 0, 0, 80, 80, 80]);
        var quadIndices:Vector.<int> = Vector.<int>([0, 1, 2, 1, 3, 2]);
        var uv:Vector.<Number> = Vector.<Number>([0, 0, 1, 0, 0, 1, 1, 1]);
        var uvt:Vector.<Number> = Vector.<Number>([0, 0, 1, 1, 0, 0.25, 0, 1, 1, 1, 1, 0.25]);
        var box:Matrix = new Matrix();
        box.createGradientBox(80, 80);
        var g:*;

        // 0. Solid fill with uv.
        g = cell("solid uv");
        g.beginFill(0xFF0000);
        g.drawTriangles(quad, quadIndices, uv);
        g.endFill();
        report();

        // 1. Linear gradient (default matrix) with uv.
        g = cell("linear default uv");
        g.beginGradientFill("linear", [0xFF0000, 0x0000FF], [1, 1], [0, 255]);
        g.drawTriangles(quad, quadIndices, uv);
        g.endFill();
        report();

        // 2. Linear gradient (default matrix) without uv.
        g = cell("linear default");
        g.beginGradientFill("linear", [0xFF0000, 0x0000FF], [1, 1], [0, 255]);
        g.drawTriangles(quad, quadIndices);
        g.endFill();
        report();

        // 3. Linear gradient (box matrix) with uv.
        g = cell("linear box uv");
        g.beginGradientFill("linear", [0xFF0000, 0x0000FF], [1, 1], [0, 255], box);
        g.drawTriangles(quad, quadIndices, uv);
        g.endFill();
        report();

        // 4. Linear gradient (box matrix) without uv.
        g = cell("linear box");
        g.beginGradientFill("linear", [0xFF0000, 0x0000FF], [1, 1], [0, 255], box);
        g.drawTriangles(quad, quadIndices);
        g.endFill();
        report();

        // 5. Radial gradient (box matrix) with uvt.
        g = cell("radial box uvt");
        g.beginGradientFill("radial", [0xFF0000, 0x0000FF], [1, 1], [0, 255], box);
        g.drawTriangles(quad, quadIndices, uvt);
        g.endFill();
        report();

        // 6. Bitmap fill without uv (fill matrix applies).
        g = cell("bitmap no uv");
        var m:Matrix = new Matrix();
        m.scale(10, 10);
        m.translate(5, 5);
        g.beginBitmapFill(bmd, m);
        g.drawTriangles(quad, quadIndices);
        g.endFill();
        report();

        // 7. Bitmap fill without uv, identity matrix.
        g = cell("bitmap no uv identity");
        g.beginBitmapFill(bmd);
        g.drawTriangles(quad, quadIndices);
        g.endFill();
        report();

        // 8. Semi-transparent solid fill with uvt.
        g = cell("solid alpha uvt");
        g.beginFill(0x00FF00, 0.5);
        g.drawTriangles(quad, quadIndices, uvt);
        g.endFill();
        report();

        // 9. Solid fill with uv, then bitmap fill without uv in the same shape.
        g = cell("solid uv then bitmap");
        g.beginFill(0xFF00FF);
        g.drawTriangles(Vector.<Number>([0, 0, 40, 0, 0, 80]), null, Vector.<Number>([0, 0, 1, 0, 0, 1]));
        g.beginBitmapFill(bmd, m);
        g.drawTriangles(Vector.<Number>([40, 0, 80, 0, 80, 80]));
        g.endFill();
        report();

        // 10. Stroke with a solid fill and uv.
        g = cell("stroke solid uv");
        g.lineStyle(4, 0x000000);
        g.beginFill(0xFF0000);
        g.drawTriangles(quad, quadIndices, uv);
        g.endFill();
        report();

        // 11. Stroke without a fill, with uv.
        g = cell("stroke no fill uv");
        g.lineStyle(4, 0x000000);
        g.drawTriangles(quad, quadIndices, uv);
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
