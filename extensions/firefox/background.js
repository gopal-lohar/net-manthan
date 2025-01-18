const NATIVE_APP_NAME = "com.netmanthan.nativemessaging";

browser.downloads.onCreated.addListener(async (downloadItem) => {
  try {
    await browser.downloads.cancel(downloadItem.id);
    console.log(`Download canceled: ${downloadItem.url}`);

    const message = {
      url: downloadItem.url,
      filename: downloadItem.filename,
      mimeType: downloadItem.mime,
    };

    sendToNativeApp(message);
  } catch (error) {
    console.error("Error handling download:", error);
  }
});

function sendToNativeApp(message) {
  const port = browser.runtime.connectNative(NATIVE_APP_NAME);

  port.postMessage(message);
  console.log("Message sent to native app:", message);

  port.onMessage.addListener((response) => {
    console.log("Response from native app:", response);
  });

  port.onDisconnect.addListener(() => {
    if (port.error) {
      console.error(
        `Native messaging host disconnected: ${port.error.message}`,
      );
    } else {
      console.log("Native messaging host disconnected normally.");
    }
  });
}
