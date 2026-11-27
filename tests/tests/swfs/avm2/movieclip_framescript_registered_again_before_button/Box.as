package {
    import flash.display.MovieClip;

    public class Box extends MovieClip {
        public var label:String;

        public function Box(label:String) {
            this.label = label;
            addFrameScript(0, frame1);
        }

        public function registerAgain():void {
            addFrameScript(0, frame1Again);
        }

        private function frame1():void {
            trace(label + ": frame 1 script");
            stop();
        }

        private function frame1Again():void {
            trace(label + ": second frame 1 script");
            stop();
        }
    }
}
