import { $, on } from "./shared.js";
import initSzdtWasm from "../vendor/szdt_wasm.js";

/** 500 MB */
const MAX_FILE_SIZE = 500 * 1024 * 1024;

const uploadFile = async (file) => {
  if (file.size > MAX_FILE_SIZE) {
    throw new Error(`File too large. Maximum size is ${MAX_FILE_SIZE}`);
  }

  // Read file as array buffer
  const arrayBuffer = await file.arrayBuffer();
  const data = new Uint8Array(arrayBuffer);
  return data;
};

const initializeFileInput = (
  fileInput,
) => {
  on(fileInput, "change", async (e) => {
    const event = e;
    const target = event.target;
    const files = target.files;
    if (!files || files.length === 0) return;
    const file = files[0];
    const bytes = await uploadFile(file);
    console.log(bytes);
  });
};

const main = async () => {
  console.log("Main initializing");

  await initSzdtWasm();

  const fileInput = $("#file-input");
  initializeFileInput(fileInput);

  $("body")?.classList.add("ready");
};

main();
