package {
import flash.display.BitmapData;
import flash.display.GraphicsBitmapFill;
import flash.display.GraphicsEndFill;
import flash.display.GraphicsTrianglePath;
import flash.display.IGraphicsData;
import flash.display.Shape;
import flash.display.Sprite;
import flash.geom.Matrix;

// Checks how uv/uvt data maps a bitmap fill onto triangles: indexed and
// sequential vertices, the fill matrix, repeat, smooth, and t values.
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
        var g:*;

        // 0. Indexed, uv 0..1.
        g = cell("indexed uv");
        g.beginBitmapFill(bmd);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([0, 0, 1, 0, 0, 1, 1, 1]));
        g.endFill();
        report();

        // 1. Sequential vertices (null indices), uv 0..1.
        g = cell("sequential uv");
        g.beginBitmapFill(bmd);
        g.drawTriangles(Vector.<Number>([0, 0, 80, 0, 0, 80, 80, 0, 80, 80, 0, 80]), null,
            Vector.<Number>([0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 1]));
        g.endFill();
        report();

        // 2. Fill matrix with uv.
        g = cell("matrix uv");
        var m:Matrix = new Matrix();
        m.scale(3, 3);
        m.rotate(0.5);
        m.translate(7, 5);
        g.beginBitmapFill(bmd, m);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([0, 0, 1, 0, 0, 1, 1, 1]));
        g.endFill();
        report();

        // 3. repeat = true, uv 0..2.
        g = cell("repeat uv 0..2");
        g.beginBitmapFill(bmd, null, true);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([0, 0, 2, 0, 0, 2, 2, 2]));
        g.endFill();
        report();

        // 4. repeat = false, uv 0..2.
        g = cell("no repeat uv 0..2");
        g.beginBitmapFill(bmd, null, false);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([0, 0, 2, 0, 0, 2, 2, 2]));
        g.endFill();
        report();

        // 5. smooth = true.
        g = cell("smooth");
        g.beginBitmapFill(bmd, null, true, true);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([0, 0, 1, 0, 0, 1, 1, 1]));
        g.endFill();
        report();

        // 6. repeat = false, uv -1..1.
        g = cell("no repeat uv -1..1");
        g.beginBitmapFill(bmd, null, false);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([-1, -1, 1, -1, -1, 1, 1, 1]));
        g.endFill();
        report();

        // 7. uvt with t = 1 everywhere.
        g = cell("uvt t=1");
        g.beginBitmapFill(bmd);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1]));
        g.endFill();
        report();

        // 8. Perspective: t = 1 on the left, 0.25 on the right.
        g = cell("uvt perspective");
        g.beginBitmapFill(bmd);
        g.drawTriangles(Vector.<Number>([0, 0, 80, 25, 0, 80, 80, 55]), quadIndices,
            Vector.<Number>([0, 0, 1, 1, 0, 0.25, 0, 1, 1, 1, 1, 0.25]));
        g.endFill();
        report();

        // 9. Same geometry, uv only (affine).
        g = cell("uv affine");
        g.beginBitmapFill(bmd);
        g.drawTriangles(Vector.<Number>([0, 0, 80, 25, 0, 80, 80, 55]), quadIndices,
            Vector.<Number>([0, 0, 1, 0, 0, 1, 1, 1]));
        g.endFill();
        report();

        // 10. uvt with t scaled uniformly by 10 (should equal t = 1).
        g = cell("uvt t=10");
        g.beginBitmapFill(bmd);
        g.drawTriangles(quad, quadIndices, Vector.<Number>([0, 0, 10, 1, 0, 10, 0, 1, 10, 1, 1, 10]));
        g.endFill();
        report();

        // 11. GraphicsTrianglePath via drawGraphicsData, uvt perspective.
        g = cell("graphics data uvt");
        g.drawGraphicsData(Vector.<IGraphicsData>([
            new GraphicsBitmapFill(bmd),
            new GraphicsTrianglePath(Vector.<Number>([0, 0, 80, 25, 0, 80, 80, 55]), quadIndices,
                Vector.<Number>([0, 0, 1, 1, 0, 0.25, 0, 1, 1, 1, 1, 0.25])),
            new GraphicsEndFill()
        ]));
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
