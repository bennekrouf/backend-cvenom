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
      // Firebase project used to validate end-user Google ID tokens.
      // Update this to your project's Firebase project ID.
      CVENOM_GOOGLE_PROJECT_ID: 'your-firebase-project-id',

      // === OPTIONAL VARIABLES (used by start.sh) ===
      DEFAULT_DOMAIN: 'keyteo.ch',
      DEFAULT_TENANT: 'keyteo'
    }
  }]
};
