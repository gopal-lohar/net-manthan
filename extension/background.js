const nativeAppName = "com.down_poc.native_messaging";

browser.downloads.onCreated.addListener((downloadItem) => {
  sendMessageToNativeApp(downloadItem);
});

function sendMessageToNativeApp(downloadItem) {
  const message = {
    action: "startDownload",
    url: downloadItem.url,
    filename: downloadItem.filename,
  };

  console.log(downloadItem);

  const port = browser.runtime.connectNative(nativeAppName);
  port.postMessage(message);

  port.onMessage.addListener((response) => {
    console.log("Received from native app:", response);
  });
}
