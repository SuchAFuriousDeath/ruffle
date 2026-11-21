package {
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Matrix3D;
    import flash.geom.Vector3D;

    public class Test extends Sprite {
        public function Test() {
            var s:Sprite;

            trace("// transform.matrix3D returns the same object while the transform is 3D");
            s = new Sprite();
            s.z = 1;
            var m:Matrix3D = s.transform.matrix3D;
            trace("same object: " + (s.transform.matrix3D == m));
            s.z = 2;
            s.x = 50;
            s.rotation = 45;
            trace("after z, x and rotation: same object " + (s.transform.matrix3D == m) + ", rawData = " + m.rawData);
            var transform:* = s.transform;
            trace("same object through another Transform: " + (transform.matrix3D == m));

            trace("");
            trace("// Changes to the object apply to the display object");
            s = new Sprite();
            s.z = 1;
            s.rotation = 30;
            m = s.transform.matrix3D;
            m.appendTranslation(10, 20, 30);
            describe("appendTranslation(10, 20, 30)", s);
            m.appendRotation(10, Vector3D.Z_AXIS);
            describe("appendRotation(10, Z_AXIS)", s);
            m.appendScale(2, 1, 1);
            describe("appendScale(2, 1, 1)", s);
            m.rawData = Vector.<Number>([3, 0, 0, 0, 0, 3, 0, 0, 0, 0, 1, 0, 7, 8, 9, 1]);
            describe("rawData = ...", s);
            m.identity();
            describe("identity()", s);

            trace("");
            trace("// Changes to the object decompose the rotation and scale again");
            s = new Sprite();
            s.z = 1;
            s.rotation = 720;
            m = s.transform.matrix3D;
            trace("rotation = 720: " + s.rotation);
            m.appendTranslation(1, 0, 0);
            trace("then appendTranslation(1, 0, 0): " + s.rotation);

            trace("");
            trace("// An assigned Matrix3D is kept");
            var assigned:Matrix3D = new Matrix3D();
            assigned.appendScale(2, 2, 2);
            s = new Sprite();
            s.transform.matrix3D = assigned;
            trace("same object: " + (s.transform.matrix3D == assigned));
            s.x = 9;
            s.scaleY = 3;
            trace("after x = 9 and scaleY = 3: rawData = " + assigned.rawData);
            assigned.appendTranslation(0, 33, 0);
            trace("after appendTranslation(0, 33, 0): y = " + s.y);
            s.transform.matrix3D = assigned;
            trace("assigned again: same object " + (s.transform.matrix3D == assigned));

            trace("");
            trace("// Going back to a 2D transform unbinds the object");
            s = new Sprite();
            s.z = 1;
            m = s.transform.matrix3D;
            s.transform.matrix = new Matrix();
            m.appendTranslation(1, 0, 0);
            trace("matrix = (identity), then appendTranslation(1, 0, 0): x = " + s.x + ", matrix3D = " + s.transform.matrix3D);
            s.z = 3;
            trace("z = 3: same object as before " + (s.transform.matrix3D == m));
            m.appendTranslation(100, 0, 0);
            trace("old object appendTranslation(100, 0, 0): x = " + s.x);

            s = new Sprite();
            s.z = 1;
            m = s.transform.matrix3D;
            s.transform.matrix3D = null;
            m.appendTranslation(1, 0, 0);
            trace("matrix3D = null, then appendTranslation(1, 0, 0): x = " + s.x + ", matrix3D = " + s.transform.matrix3D);

            trace("");
            trace("// Copies aren't bound");
            s = new Sprite();
            s.z = 1;
            var copy:Matrix3D = s.transform.matrix3D.clone();
            copy.appendTranslation(50, 0, 0);
            trace("clone().appendTranslation(50, 0, 0): x = " + s.x);
        }

        private function describe(label:String, s:Sprite):void {
            trace(label + ": x=" + s.x + " y=" + s.y + " z=" + s.z + " rotation=" + s.rotation + " scaleX=" + s.scaleX + " scaleY=" + s.scaleY);
        }
    }
}
