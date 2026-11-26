package {
import flash.display.Shape;
import flash.display.Sprite;

// Checks whether drawTriangles culling uses local or screen-space winding.
// Red triangles have positive local winding, blue ones negative. Each
// transform is drawn with "positive" and "negative" culling.
[SWF(width="400", height="300", backgroundColor="#FFFFFF")]
public class Test extends Sprite {
    private var cellId:int = 0;

    public function Test() {
        for each (var culling:String in ["positive", "negative"]) {
            draw("none", culling, 1, 1, 1, 0);
            draw("shape scaleX -1", culling, -1, 1, 1, 0);
            draw("parent scaleX -1", culling, 1, 1, -1, 0);
            draw("shape scaleY -1", culling, 1, -1, 1, 0);
            draw("rotation 180", culling, 1, 1, 1, 180);
            draw("shape and parent scaleX -1", culling, -1, 1, -1, 0);
        }

        trace("Done");
    }

    private function draw(name:String, culling:String, sx:Number, sy:Number, parentSx:Number, rotation:Number):void {
        var parent:Sprite = new Sprite();
        parent.x = (cellId % 4) * 100 + 50;
        parent.y = Math.floor(cellId / 4) * 100 + 50;
        parent.scaleX = parentSx;
        addChild(parent);

        var s:Shape = new Shape();
        s.scaleX = sx;
        s.scaleY = sy;
        s.rotation = rotation;
        parent.addChild(s);

        // Positive orientation ((b - a) x (c - a) > 0 in y-down local space).
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawTriangles(Vector.<Number>([-35, -35, -5, -35, -35, 35]), null, null, culling);
        s.graphics.endFill();

        // Negative orientation.
        s.graphics.beginFill(0x0000FF);
        s.graphics.drawTriangles(Vector.<Number>([5, -35, 5, 35, 35, -35]), null, null, culling);
        s.graphics.endFill();

        trace(cellId + " " + name + " " + culling + ": rect " + s.getRect(s));
        ++cellId;
    }
}
}
