package {
    import flash.display.MovieClip;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {
        public function Test() {
            super();
            var con:NetConnection = new NetConnection();
            con.connect(null);
            var stream:NetStream = new NetStream(con);
            stream.client = {};
            stream.play(null);

            var ba:ByteArray = new ByteArray();
            ba.writeUTFBytes("HelloFLV");

            // Start with position at 2 to verify only bytes from the current
            // position forward are appended.
            ba.position = 2;

            trace("before length:         " + ba.length);
            trace("before position:       " + ba.position);
            trace("before bytesAvailable: " + ba.bytesAvailable);

            stream.appendBytes(ba);

            // Flash does not advance the ByteArray's position after
            // appendBytes - verify Ruffle matches.
            trace("after length:          " + ba.length);
            trace("after position:        " + ba.position);
            trace("after bytesAvailable:  " + ba.bytesAvailable);

            // Reset and verify position stays at 0 after a full-buffer append.
            ba.position = 0;
            stream.appendBytes(ba);
            trace("second position:       " + ba.position);

            // Empty append: position already past end, nothing to read.
            ba.position = ba.length;
            stream.appendBytes(ba);
            trace("empty position:        " + ba.position);

            trace("test over");
        }
    }
}
