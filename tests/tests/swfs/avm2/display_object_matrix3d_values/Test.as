package {
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Matrix3D;

    public class Test extends Sprite {
        public function Test() {
            var s:Sprite;

            trace("// The rotation of a 3D transform is kept as it is");
            for each (var rotation:Number in [0.1, 200, -200, 200.123456789, 720, 1e10, NaN]) {
                s = new Sprite();
                s.z = 1;
                s.rotation = rotation;
                trace("rotation = " + rotation + ": " + s.rotation + ", rawData = " + s.transform.matrix3D.rawData.slice(0, 6));
            }

            trace("");
            trace("// The scale of a 3D transform is kept at f32 precision, and not 0");
            for each (var scale:Number in [0.1, 2.5, -7.77, 1.0000001, 1e20, 0, -1e-7, 1e-40, NaN, Infinity]) {
                s = new Sprite();
                s.z = 1;
                s.scaleX = scale;
                s.scaleY = scale;
                trace("scale = " + scale + ": " + s.scaleX + " " + s.scaleY + ", rawData = " + s.transform.matrix3D.rawData.slice(0, 6));
            }

            trace("");
            trace("// The translation of a 3D transform is kept in twips at f32 precision");
            for each (var translation:Number in [0.1, 0.33, 1.23456789, -7.77, 0.01, 12345.678, NaN, Infinity]) {
                s = new Sprite();
                s.z = translation;
                s.x = translation;
                s.y = translation;
                trace("x = y = z = " + translation + ": " + s.x + " " + s.y + " " + s.z + ", rawData = " + s.transform.matrix3D.rawData.slice(12, 15));

                var m:Matrix3D = new Matrix3D();
                m.appendTranslation(translation, translation, translation);
                s = new Sprite();
                s.transform.matrix3D = m;
                trace("  assigned: " + s.x + " " + s.y + " " + s.z + ", rawData = " + s.transform.matrix3D.rawData.slice(12, 15));
            }

            trace("");
            trace("// Setting matrix to null switches to a 3D transform");
            s = new Sprite();
            s.x = 5;
            s.scaleX = 2;
            s.transform.matrix = null;
            trace("2D: x=" + s.x + " z=" + s.z + " scaleX=" + s.scaleX + " matrix=" + s.transform.matrix + " rawData=" + s.transform.matrix3D.rawData);
            s = new Sprite();
            s.z = 7;
            s.transform.matrix = null;
            trace("3D: x=" + s.x + " z=" + s.z + " scaleX=" + s.scaleX + " matrix=" + s.transform.matrix + " rawData=" + s.transform.matrix3D.rawData);

            trace("");
            trace("// width and height of a 3D transform");
            s = new Sprite();
            s.graphics.beginFill(0);
            s.graphics.drawRect(0, 0, 100, 50);
            s.z = 1;
            trace("width=" + s.width + " height=" + s.height);
            s.width = 200;
            trace("width = 200: scaleX=" + s.scaleX + " width=" + s.width + " 3D=" + (s.transform.matrix3D != null));
        }
    }
}
