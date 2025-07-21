/// <reference lib="dom" />
import { $, on } from "./dom.ts";
import initSzdtWasm from "../vendor/szdt_wasm/szdt_wasm.js";

/** 500 MB */
export const MAX_FILE_SIZE = 500 * 1024 * 1024;

export const uploadFile = async (file: File): Promise<Uint8Array> => {
  if (file.size > MAX_FILE_SIZE) {
    throw new Error(`File too large. Maximum size is ${MAX_FILE_SIZE}`);
  }

  // Read file as array buffer
  const arrayBuffer = await file.arrayBuffer();
  const data = new Uint8Array(arrayBuffer);
  return data;
};

export const initializeFileInput = (
  fileInput: HTMLInputElement,
): void => {
  on(fileInput, "change", async (e) => {
    const event = e as Event;
    const target = event.target as HTMLInputElement;
    const files = target.files;
    if (!files || files.length === 0) return;
    const file = files[0];
    const bytes = await uploadFile(file);
    console.log(bytes);
  });
};

export const main = async () => {
  console.log("Main initializing");

  await initSzdtWasm();

  const fileInput = $("#file-input") as HTMLInputElement;
  initializeFileInput(fileInput);

  $("body")?.classList.add("ready");
};

main();
