import.meta.env.PROD;

const isProd = import.meta.env.PROD;

export const API_URL = isProd ? "/api" : "http://localhost:3000/api";

export const WS_URL = isProd
  ? `wss://${window.location.host}/api`
  : "ws://localhost:3000/api";
