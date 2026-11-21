function keys(obj) {
    var result = [];
    for (var key in obj) {
        result.push(key);
    }
    result.sort();
    return result.join(",");
}

var fr = new flash.net.FileReference();
trace("FileReference: " + keys(fr));
var frl = new flash.net.FileReferenceList();
trace("FileReferenceList: " + keys(frl));
trace("frl.fileList: " + typeof frl.fileList + " " + frl.fileList + ", own: " + frl.hasOwnProperty("fileList"));
trace("fr.name own: " + fr.hasOwnProperty("name") + ", proto: " + flash.net.FileReference.prototype.hasOwnProperty("name"));
var fr2 = new flash.net.FileReference();
fr2.name = "changed";
trace("fr2.name after set: " + fr2.name);
fr2.postData = "pd";
trace("fr2.postData after set: " + fr2.postData);
trace("delete fr2.name: " + delete fr2.name + ", own after: " + fr2.hasOwnProperty("name"));
var frl2 = new flash.net.FileReferenceList();
trace("frl2.fileList.length: " + frl2.fileList.length + ", instanceof Array: " + (frl2.fileList instanceof Array));
trace("delete frl2.fileList: " + delete frl2.fileList + ", own after: " + frl2.hasOwnProperty("fileList"));
trace("two lists share fileList: " + (frl.fileList == frl2.fileList));
var fr3 = new flash.net.FileReference();
trace("new postData: " + typeof fr3.postData + " [" + fr3.postData + "]");
