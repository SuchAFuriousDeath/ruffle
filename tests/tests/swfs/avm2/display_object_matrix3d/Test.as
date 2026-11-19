package {
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Matrix3D;
    import flash.geom.Vector3D;

    public class Test extends Sprite {
        public function Test() {
            var s:Sprite;

            trace("// Setting z switches to a 3D transform, even to 0");
            for each (var z:Number in [0, 30]) {
                s = new Sprite();
                s.x = 10;
                s.y = 20;
                s.scaleX = 2;
                s.z = z;
                describe("z = " + z, s);
            }

            trace("");
            trace("// An assigned matrix3D keeps all its cells, including the perspective ones");
            s = new Sprite();
            s.transform.matrix3D = matrixWithPerspective();
            // Flash Player decomposes matrices with perspective differently, so
            // only check the matrix and the translation here.
            describeMatrix("matrix3D with perspective", s);

            trace("");
            trace("// x, y and z only change the translation of a 3D transform");
            s = new Sprite();
            s.transform.matrix3D = matrixWithPerspective();
            s.x = 999;
            s.y = 888;
            s.z = 555;
            describeMatrix("x = 999, y = 888, z = 555", s);
            s = new Sprite();
            s.z = 1;
            s.x = 0.1;
            describe("z = 1, then x = 0.1", s);

            trace("");
            trace("// The 2D properties are decomposed from an assigned matrix3D");
            var m:Matrix3D = new Matrix3D();
            m.appendScale(-1, 1, 1);
            s = new Sprite();
            s.transform.matrix3D = m;
            describe("matrix3D = scale(-1, 1, 1)", s);
            m = new Matrix3D();
            m.appendScale(2, 4, 1);
            m.appendRotation(90, Vector3D.Z_AXIS);
            s = new Sprite();
            s.transform.matrix3D = m;
            describe("matrix3D = scale(2, 4, 1), then a rotation of 90 around Z", s);

            trace("");
            trace("// Setting the 2D properties of a 3D transform");
            s = new Sprite();
            s.z = 5;
            s.rotation = 45;
            describe("z = 5, then rotation = 45", s);
            s = new Sprite();
            s.z = 5;
            s.scaleX = -1;
            s.scaleY = 3;
            describe("z = 5, then scaleX = -1, scaleY = 3", s);

            trace("");
            trace("// Going back to a 2D transform");
            s = new Sprite();
            s.z = 5;
            s.z = 0;
            describe("z = 5, then z = 0", s);
            s = new Sprite();
            s.z = 5;
            s.transform.matrix = new Matrix(2, 0, 0, 2, 3, 4);
            describe("z = 5, then matrix = (2, 0, 0, 2, 3, 4)", s);
            s = new Sprite();
            s.transform.matrix3D = matrixWithPerspective();
            s.transform.matrix3D = null;
            describe("matrix3D with perspective, then matrix3D = null", s);
        }

        private function matrixWithPerspective():Matrix3D {
            var m:Matrix3D = new Matrix3D();
            m.copyRawDataFrom(new <Number>[
                2, 0, 0, -3,
                0, 3, 0, -6,
                0, 0, 4, -9,
                100, 200, 300, 1
            ]);
            return m;
        }

        private function describe(label:String, s:Sprite):void {
            describeMatrix(label, s);
            trace("  rotation=" + s.rotation + " scaleX=" + s.scaleX + " scaleY=" + s.scaleY);
        }

        private function describeMatrix(label:String, s:Sprite):void {
            trace(label + ":");
            trace("  x=" + s.x + " y=" + s.y + " z=" + s.z);
            trace("  matrix=" + s.transform.matrix);
            var m:Matrix3D = s.transform.matrix3D;
            trace("  matrix3D=" + (m ? m.rawData : null));
        }
    }
}
