// Tests which frame scripts registered with addFrameScript run after a
// SimpleButton whose states contain movie clips is constructed during the
// frame script phase. In each of the two frames, the main timeline's frame
// script sends "clip" to another frame and then creates boxes and a holder
// with such a button in different orders. Each tick traces the current frame
// of each box and holder, and the frame scripts that ran.
package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Test extends MovieClip {
        public var clip:MovieClip;
        private var objects:Array = [];
        private var ticks:int = 0;

        public function Test() {
            addFrameScript(0, frame1, 1, frame2);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function toggleClip():void {
            var frame:int = clip.currentFrame == 1 ? 2 : 1;
            trace("  running clip.gotoAndStop(" + frame + ")");
            clip.gotoAndStop(frame);
        }

        private function frame1():void {
            trace("V1 (goto; Holder with SimpleButton containing Kid in up+over states; A B)");
            toggleClip();
            objects.push(new Holder("V1.H"));
            objects.push(new Box("V1.A"));
            objects.push(new Box("V1.B"));
            trace("V1 end");
        }

        private function frame2():void {
            stop();
            trace("V2 (goto; Box A; Holder; Box B)");
            toggleClip();
            objects.push(new Box("V2.A"));
            objects.push(new Holder("V2.H"));
            objects.push(new Box("V2.B"));
            trace("V2 end");
        }

        private function onEnterFrame(e:Event):void {
            ticks++;

            var parts:Array = [];

            for each (var object:* in objects) {
                parts.push(object.label + "=" + object.currentFrame);
            }

            Kid.traceRan();
            trace("tick " + ticks + ": " + parts.join(", "));

            if (ticks == 8) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                trace("Done");
            }
        }
    }
}
