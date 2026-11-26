package {
import flash.display.BitmapData;
import flash.display.Shape;
import flash.display.Sprite;

// Checks whether lineStyle strokes are drawn around triangles from
// drawTriangles, with and without fills and uvtData.
[SWF(width="400", height="300", backgroundColor="#FFFFFF")]
public class Test extends Sprite {
    private var cellId:int = 0;
    private var bmd:BitmapData;

    public function Test() {
        bmd = new BitmapData(4, 4, false, 0);
        for (var y:int = 0; y < 4; y++) {
            for (var x:int = 0; x < 4; x++) {
                bmd.setPixel(x, y, ((x * 80) << 16) | ((y * 80) << 8) | 0xC0);
            }
        }

        var quad:Vector.<Number> = Vector.<Number>([10, 10, 70, 10, 10, 70, 70, 70]);
        var quadIndices:Vector.<int> = Vector.<int>([0, 1, 2, 1, 3, 2]);
        var quadUv:Vector.<Number> = Vector.<Number>([0, 0, 1, 0, 0, 1, 1, 1]);
        var seq:Vector.<Number> = Vector.<Number>([10, 10, 70, 10, 10, 70, 70, 10, 70, 70, 10, 70]);
        var g:*;

        // 0. Solid fill, indexed, semi-transparent stroke.
        g = cell("solid indexed");
        g.lineStyle(6, 0x000000, 0.5);
        g.beginFill(0xFF0000);
        g.drawTriangles(quad, quadIndices);
        g.endFill();
        report();

        // 1. Bitmap fill with uv.
        g = cell("bitmap uv");
        g.lineStyle(6, 0x000000, 0.5);
        g.beginBitmapFill(bmd);
        g.drawTriangles(quad, quadIndices, quadUv);
        g.endFill();
        report();

        // 2. Solid fill, sequential vertices.
        g = cell("solid sequential");
        g.lineStyle(6, 0x000000, 0.5);
        g.beginFill(0xFF0000);
        g.drawTriangles(seq);
        g.endFill();
        report();

        // 3. Stroke only, no fill.
        g = cell("stroke only");
        g.lineStyle(6, 0x000000, 0.5);
        g.drawTriangles(quad, quadIndices);
        report();

        // 4. No fill, no stroke.
        g = cell("nothing");
        g.drawTriangles(quad, quadIndices);
        report();

        // 5. Stroke with culling: one triangle culled.
        g = cell("culled");
        g.lineStyle(6, 0x000000, 0.5);
        g.beginFill(0xFF0000);
        g.drawTriangles(Vector.<Number>([10, 10, 70, 10, 10, 70, 70, 10, 10, 70, 70, 70]), null, null, "positive");
        g.endFill();
        report();

        // 6. Stroke path before and after drawTriangles.
        g = cell("path around");
        g.lineStyle(4, 0x0000FF);
        g.beginFill(0xFF0000);
        g.moveTo(0, 0);
        g.lineTo(80, 0);
        g.drawTriangles(Vector.<Number>([20, 30, 60, 30, 40, 60]));
        g.lineTo(80, 80);
        g.endFill();
        report();

        // 7. Degenerate triangle with stroke.
        g = cell("degenerate");
        g.lineStyle(6, 0x000000, 0.5);
        g.beginFill(0xFF0000);
        g.drawTriangles(Vector.<Number>([10, 10, 70, 70, 40, 40]));
        g.endFill();
        report();

        // 8. lineStyle set after beginFill.
        g = cell("line after fill");
        g.beginFill(0x00FF00);
        g.lineStyle(6, 0x000000, 0.5);
        g.drawTriangles(quad, quadIndices);
        g.endFill();
        report();

        // 9. Gradient line style.
        g = cell("line gradient");
        g.lineStyle(6);
        g.lineGradientStyle("linear", [0xFF0000, 0x0000FF], [1, 1], [0, 255]);
        g.beginFill(0xFFFF00);
        g.drawTriangles(quad, quadIndices);
        g.endFill();
        report();

        // 10. Stroke, no endFill, then more drawTriangles calls.
        g = cell("two calls");
        g.lineStyle(6, 0x000000, 0.5);
        g.beginFill(0xFF0000);
        g.drawTriangles(Vector.<Number>([10, 10, 70, 10, 10, 70]));
        g.drawTriangles(Vector.<Number>([70, 10, 70, 70, 10, 70]));
        g.endFill();
        report();

        // 11. Hairline stroke (thickness 0).
        g = cell("hairline");
        g.lineStyle(0, 0x000000);
        g.beginFill(0xFF0000);
        g.drawTriangles(quad, quadIndices);
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
        trace(currentName + ": bounds " + current.getBounds(current) + ", rect " + current.getRect(current) +
            ", hit(40,40) " + current.hitTestPoint(current.x + 40, current.y + 40, true) +
            ", hit(10,40) " + current.hitTestPoint(current.x + 10, current.y + 40, true) +
            ", hit(8,40) " + current.hitTestPoint(current.x + 8, current.y + 40, true));
    }
}
}
