function escape(s) {
    return s.split("\r").join("\\r").split("\n").join("\\n");
}

_root.createTextField("tf", 1, 0, 0, 200, 100);
_root.tf.type = "input";
_root.tf.multiline = true;
_root.tf.border = true;
_root.tf.restrict = "A-Za-z0-9";

_root.tf.onChanged = function() {
    trace("changed: " + escape(_root.tf.text));
};

Selection.setFocus(_root.tf);
