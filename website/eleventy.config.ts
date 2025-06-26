import { type EleventyConfig } from "@11ty/eleventy";

export default (config: EleventyConfig): void => {
  config.setOutputDirectory("../docs");
  config.addPassthroughCopy("assets");
  config.addPassthroughCopy("media");
};
