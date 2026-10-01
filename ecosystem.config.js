const path = require('path');

module.exports = {
  apps: [{
    name: 'cvenom',
    script: './start.sh',
    instances: 1,
    autorestart: true,
    watch: false,
    max_memory_restart: '1G',
    env: {
      RUST_LOG: 'info',
      ROCKET_SECRET_KEY: 'KeQu6g9OeNcF5JwvAKTTS6JDTG8lgP3RrGkw1icsEW4=',

      // === MANDATORY CVENOM ENVIRONMENT VARIABLES ===
      LOG_PATH_CVENOM: '/var/log/cvenom.log',
      ROCKET_PORT: '4002',
      CV_SERVICE_URL: 'http://localhost:5555',
      CVENOM_TENANT_DATA_PATH: '/var/cvenom/tenant-data',
      CVENOM_OUTPUT_PATH: '/var/cvenom/output',
      CVENOM_TEMPLATES_PATH: path.resolve(__dirname, 'templates'),
      CVENOM_DATABASE_PATH: '/var/cvenom/tenants.db',
      JOB_MATCHING_API_URL: 'http://127.0.0.1:5555',
      SERVICE_TIMEOUT: '30000',

      // === GOOGLE / FIREBASE AUTH ===
      // Firebase project used to validate end-user Google ID tokens (browser/mobile path).
      // Update this to your project's Firebase project ID.
      CVENOM_GOOGLE_PROJECT_ID: 'your-firebase-project-id',

      // OIDC audience for api0.ai gateway service-account tokens.
      // Leave unset to disable the OIDC path (browser Firebase auth still works).
      // api0 signs as itself, with an audience scoped to the cvenom tenant.
      // On the VPS the live value is in /opt/cvenom/backend-cvenom.env (run-backend.sh
      // loads it and it overrides this file).
      CVENOM_OIDC_AUDIENCE: 'https://api.cvenom.com/api0/tenant/c752f139-9fad-47c9-b10b-9983dcce8333',

      // Who may sign those tokens — api0's service account. Required whenever
      // CVENOM_OIDC_AUDIENCE is set: any Google service account can mint a token
      // for any audience, so without this every api0 request is refused.
      // Comma-separate two during a key rotation.
      CVENOM_OIDC_SERVICE_ACCOUNT: 'firebase-adminsdk-fbsvc@cvenom-de582.iam.gserviceaccount.com',

      // === OPTIONAL VARIABLES (used by start.sh) ===
      DEFAULT_DOMAIN: 'keyteo.ch',
      DEFAULT_TENANT: 'keyteo'
    }
  }]
};
