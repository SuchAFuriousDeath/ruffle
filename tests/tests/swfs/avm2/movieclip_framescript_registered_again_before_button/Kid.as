package {
    import flash.display.MovieClip;

    public class Kid extends MovieClip {
        public static var count:int = 0;

        public var id:int;

        public function Kid() {
            id = ++count;
            trace("  Kid#" + id + " ctor");
            addFrameScript(0, frame1);
        }

        private function frame1():void {
            stop();
        }
    }
}
