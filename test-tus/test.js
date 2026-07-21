const fs = require('fs');
const tus = require('tus-js-client');

// Create a 5MB dummy file
const filePath = 'dummy.mp4';
const size = 5 * 1024 * 1024;
const buffer = Buffer.alloc(size, 'a');
fs.writeFileSync(filePath, buffer);

const file = fs.createReadStream(filePath);

const options = {
  endpoint: 'http://127.0.0.1:1081/files/',
  retryDelays: [0, 3000, 5000, 10000, 20000],
  metadata: {
    filename: 'dummy.mp4',
    filetype: 'video/mp4',
    userId: '1'
  },
  onError(error) {
    console.error('Upload failed:', error);
    process.exit(1);
  },
  onProgress(bytesUploaded, bytesTotal) {
    const percentage = ((bytesUploaded / bytesTotal) * 100).toFixed(2);
    console.log(bytesUploaded, bytesTotal, percentage + '%');
  },
  onSuccess() {
    console.log('Upload completed successfully!');
    console.log('Upload URL:', upload.url);
    process.exit(0);
  },
};

const upload = new tus.Upload(file, {
  ...options,
  uploadSize: size
});

console.log('Starting TUS upload...');
upload.start();
