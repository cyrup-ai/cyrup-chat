# OAuth Setup Guide

The CYRUP Chat app uses OAuth 2.0 for authentication with Google and GitHub.

## Quick Start

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Follow the setup instructions below for your preferred OAuth provider

3. Run the app:
   ```bash
   dx serve
   ```

## Google OAuth Setup

### 1. Create OAuth Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. Create a new project or select an existing one
3. Click **"Create Credentials"** → **"OAuth client ID"**
4. Select **"Desktop application"** as the application type
5. Name it (e.g., "CYRUP Chat Desktop")
6. Click **"Create"**

### 2. Configure Redirect URI

1. After creating, click on your OAuth client
2. Add **`http://localhost:8080`** to **Authorized redirect URIs**
3. Click **"Save"**

### 3. Copy Credentials

1. Copy the **Client ID** and **Client secret**
2. Add them to your `.env` file:
   ```bash
   GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
   GOOGLE_CLIENT_SECRET=your-client-secret
   ```

## GitHub OAuth Setup (Optional)

### 1. Create OAuth App

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click **"New OAuth App"**
3. Fill in the details:
   - **Application name:** CYRUP Chat
   - **Homepage URL:** http://localhost:8080
   - **Authorization callback URL:** http://localhost:8080
4. Click **"Register application"**

### 2. Generate Client Secret

1. On the app page, click **"Generate a new client secret"**
2. Copy the **Client ID** and **Client secret**
3. Add them to your `.env` file:
   ```bash
   GITHUB_CLIENT_ID=your-github-client-id
   GITHUB_CLIENT_SECRET=your-github-client-secret
   ```

## How OAuth Works in CYRUP

1. User clicks "Sign in with Google" or "Sign in with GitHub"
2. A local HTTP server starts on port 8080
3. System browser opens to OAuth consent page
4. User approves access
5. Browser redirects to http://localhost:8080 with auth code
6. App exchanges code for access token
7. App fetches user profile information
8. User is logged in!

## Troubleshooting

### Error: "OAuth credentials not configured"

**Solution:** You haven't set up the environment variables. Follow the setup guide above.

### Error: "Address already in use (port 8080)"

**Solution:** Another application is using port 8080. Either:
- Stop the other application
- Or change the OAuth redirect port (requires code modification)

### Browser doesn't open

**Solution:** On macOS, ensure you have the `open` command available:
```bash
which open
```

### "Invalid redirect URI" error

**Solution:** Make sure you added `http://localhost:8080` exactly as shown in your OAuth app settings.

## Security Notes

- Never commit your `.env` file to version control
- The `.env` file is already in `.gitignore`
- OAuth credentials are stored securely in memory using `Zeroizing<String>`
- Tokens are automatically cleared from memory when dropped
