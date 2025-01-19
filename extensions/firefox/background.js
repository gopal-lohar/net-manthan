let port = browser.runtime.connectNative("com.net.manthan");

port.onDisconnect.addListener((p) => {
  if (p.error) {
    console.error(`Disconnected due to error: ${p.error.message}`);
  }
});

browser.downloads.onCreated.addListener(async (downloadItem) => {
  await browser.downloads.cancel(downloadItem.id);
  console.log(downloadItem);
  console.log(`Download\n\n ${JSON.stringify(downloadItem)} \n\n\n`);
  const message = {
    url: downloadItem.url,
    filename: downloadItem.filename,
    mime: downloadItem.mime,
    referrer: downloadItem.referrer,
    headers: downloadItem.headers,
  };
  console.log(message);
  port.postMessage(message);
});

port.onMessage.addListener((response) => {
  console.log("Received from native app:", response);
});
