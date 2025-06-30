const isProduction = process.env.NODE_ENV === "production";

export default {
  url: isProduction ? "/szdt/" : "/",
};
