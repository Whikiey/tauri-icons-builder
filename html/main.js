const { invoke } = window.__TAURI__.tauri;

async function browse_png() {
  var path = await invoke("browse_png");
  $("#image_path").val(path);
  var png_base64 = await invoke("get_web_img_png_base64", { imagePath: path });
  $("#logo").attr("src", "data:image/png;base64," + png_base64);
}

async function browse_dir() {
  var dir = await invoke("browse_dir");
  $("#target_dir").val(dir);
}

async function build_icons(imagePath, targetDir) {
  $("#msg").text("正在生成图标集，请稍等大约几秒钟...");
  await invoke("build_icons", { imagePath: imagePath, targetDir: targetDir });
  $("#msg").text("完成");
}

$(function() {
  $("#browse_png").on("click", function() {
    console.log("browse_png");
    browse_png();
    return false;
  });

  $("#browse_dir").on("click", function() {
    console.log("browse_dir");
    browse_dir();
    return false;
  });

  $("#build_icons").on("click", function() {
    console.log("build_icons");
    build_icons($("#image_path").val(), $("#target_dir").val());
    return false;
  });
  
});