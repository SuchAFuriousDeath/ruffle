package {
import flash.display.Shape;
import flash.display.Sprite;
import flash.geom.Rectangle;

// Checks what drawTriangles does to a fill path that is still open when it
// is called. Each cell is clipped with a scrollRect, so a fill that is no
// longer closed stays inside its cell.
[SWF(width="400", height="300", backgroundColor="#FFFFFF")]
public class Test extends Sprite {
    private var cellId:int = 0;
    private var far:Vector.<Number> = Vector.<Number>([0, 85, 10, 85, 0, 95]);

    public function Test() {
        var g:*;

        // 0. Baseline without drawTriangles.
        g = cell("baseline");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.endFill();

        // 1. Triangles after two edges, then endFill.
        g = cell("triangles then endFill");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.drawTriangles(far);
        g.endFill();

        // 2. Triangles after two edges, then one more edge.
        g = cell("triangles then lineTo");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.drawTriangles(far);
        g.lineTo(10, 70);
        g.endFill();

        // 3. Empty drawTriangles call.
        g = cell("empty triangles");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.drawTriangles(new Vector.<Number>());
        g.lineTo(10, 70);
        g.endFill();

        // 4. All triangles culled.
        g = cell("culled triangles");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.drawTriangles(Vector.<Number>([0, 85, 10, 85, 0, 95]), null, null, "positive");
        g.lineTo(10, 70);
        g.endFill();

        // 5. Triangles, then moveTo and a new closed path.
        g = cell("triangles then moveTo");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.drawTriangles(far);
        g.moveTo(40, 40);
        g.lineTo(60, 40);
        g.lineTo(60, 60);
        g.endFill();

        // 6. Closed subpath before the triangles.
        g = cell("closed then triangles");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.lineTo(10, 10);
        g.drawTriangles(far);
        g.lineTo(10, 70);
        g.endFill();

        // 7. Nonzero winding path via drawPath, open, then triangles.
        g = cell("drawPath nonZero open");
        g.beginFill(0xFF0000, 0.5);
        g.drawPath(Vector.<int>([1, 2, 2]), Vector.<Number>([10, 10, 70, 10, 70, 70]), "nonZero");
        g.drawTriangles(far);
        g.endFill();

        // 8. Solid lineStyle along the open path.
        g = cell("stroke open path");
        g.lineStyle(3, 0x0000FF);
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.drawTriangles(far);
        g.lineTo(10, 70);
        g.endFill();

        // 9. Triangles, then beginFill with another color (implicit endFill).
        g = cell("triangles then beginFill");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.lineTo(70, 70);
        g.drawTriangles(far);
        g.beginFill(0x0000FF, 0.5);
        g.drawRect(40, 40, 20, 20);
        g.endFill();

        // 10. Triangles then a curveTo.
        g = cell("triangles then curveTo");
        g.beginFill(0xFF0000, 0.5);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.drawTriangles(far);
        g.curveTo(70, 70, 10, 70);
        g.endFill();

        // 11. No fill; open stroke path, triangles, more stroke.
        g = cell("stroke only");
        g.lineStyle(3, 0x0000FF);
        g.moveTo(10, 10);
        g.lineTo(70, 10);
        g.drawTriangles(far);
        g.lineTo(70, 70);

        trace("Done");
    }

    private function cell(name:String):* {
        var clip:Sprite = new Sprite();
        clip.x = (cellId % 4) * 100;
        clip.y = Math.floor(cellId / 4) * 100;
        clip.scrollRect = new Rectangle(0, 0, 98, 98);
        addChild(clip);

        var s:Shape = new Shape();
        clip.addChild(s);
        ++cellId;
        return s.graphics;
    }
}
}
