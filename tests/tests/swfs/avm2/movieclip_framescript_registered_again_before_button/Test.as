// Tests when a frame script runs that is registered with addFrameScript
// during the frame script phase for the current frame of a clip whose frame
// script already ran. In frame 2 of the main timeline, nothing else happens
// afterwards. In frame 3, a SimpleButton whose states contain movie clips is
// constructed afterwards. Each tick traces the current frame of both clips.
package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Test extends MovieClip {
        private var x1:Box;
        private var x2:Box;
        private var ticks:int = 0;

        public function Test() {
            addFrameScript(0, frame1, 1, frame2, 2, frame3);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function frame1():void {
            trace("root frame 1");
            x1 = new Box("X1");
            x2 = new Box("X2");
            trace("root frame 1 end");
        }

        private function frame2():void {
            trace("root frame 2: X1.registerAgain()");
            x1.registerAgain();
            trace("root frame 2 end");
        }

        private function frame3():void {
            stop();
            trace("root frame 3: X2.registerAgain(), then new Holder");
            x2.registerAgain();
            new Holder("H");
            trace("root frame 3 end");
        }

        private function onEnterFrame(e:Event):void {
            ticks++;
            trace("tick " + ticks + ": X1=" + x1.currentFrame + ", X2=" + x2.currentFrame);

            if (ticks == 5) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                trace("Done");
            }
        }
    }
}
