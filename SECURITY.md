# Security Guidelines

## 🔒 Credential Management

**IMPORTANT:** Never commit passwords, API keys, or secrets to version control!

### ✅ Safe Practices

1. **Use Environment Variables**
   ```bash
   # Create .env.test from example
   cp .env.test.example .env.test

   # Edit with your credentials
   nano .env.test

   # Load variables
   source .env.test

   # Run scripts
   ./get-events-safe.sh
   ```

2. **Pass as Command Arguments**
   ```bash
   # Scripts accept password as parameter
   ./get-all-events.sh admin 'your-password' https://api-8004-dev.fly.dev

   # Or with environment variable
   export API_PASSWORD='your-password'
   ./test-endpoints.sh
   ```

3. **Use .gitignore**
   ```bash
   # These files are ignored by git:
   .env
   .env.local
   .env.test
   *.secret
   *.key
   ```

### ❌ Unsafe Practices

- ❌ Hardcoding passwords in scripts
- ❌ Committing `.env` files
- ❌ Sharing credentials in chat/email
- ❌ Using same password in multiple environments

## 🔐 API Security

### Production Deployment

1. **Use Strong Passwords**
   - Minimum 32 characters
   - Mix of letters, numbers, symbols
   - Generate with: `openssl rand -base64 32`

2. **Use Password Hashing**
   ```bash
   # Generate bcrypt hash
   cargo run --bin generate_password_hash

   # Set in Fly.io secrets
   flyctl secrets set AUTH_PASSWORD_HASH="$2b$12$..." --app api-8004-dev
   ```

3. **Rotate Secrets Regularly**
   ```bash
   # Update JWT secret
   flyctl secrets set JWT_SECRET="$(openssl rand -base64 32)" --app api-8004-dev
   ```

4. **Limit CORS Origins**
   ```bash
   # Set specific domains
   flyctl secrets set CORS_ALLOWED_ORIGINS="https://yourdomain.com" --app api-8004-dev
   ```

## 🛡️ Access Control

### Fly.io Secrets

Sensitive data is stored in Fly.io secrets (never in code):

```bash
# View secrets (only shows names, not values)
flyctl secrets list --app api-8004-dev

# Set secrets
flyctl secrets set \
  JWT_SECRET="your-secret-key" \
  AUTH_PASSWORD_HASH="bcrypt-hash" \
  --app api-8004-dev
```

### Local Development

For local development, use `.env` file:

```env
# .env (DO NOT COMMIT)
RPC_URL=https://...
DATABASE_URL=postgresql://...
JWT_SECRET=dev-secret-min-32-chars
AUTH_PASSWORD=dev-password
```

## 📝 Checklist Before Pushing

Before pushing to GitHub, verify:

- [ ] No passwords in source files
- [ ] No API keys hardcoded
- [ ] `.env` files in `.gitignore`
- [ ] `.env.example` has placeholders only
- [ ] Secrets use Fly.io secrets manager
- [ ] Test scripts require password as parameter

### Quick Check

```bash
# Search for potential secrets
grep -r "password.*=" --include="*.sh" --include="*.js" --include="*.py" . | grep -v "PASSWORD="

# Check .gitignore
cat .gitignore | grep -E "(\.env|secret|key)"
```

## 🚨 If Secrets Are Exposed

If you accidentally commit secrets:

1. **Rotate Immediately**
   ```bash
   # Change password on Fly.io
   flyctl secrets set AUTH_PASSWORD="new-password" --app api-8004-dev

   # Change JWT secret
   flyctl secrets set JWT_SECRET="$(openssl rand -base64 32)" --app api-8004-dev
   ```

2. **Remove from Git History**
   ```bash
   # Use git-filter-repo or BFG Repo-Cleaner
   # See: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository
   ```

3. **Force Push**
   ```bash
   git push origin main --force
   ```

## 📚 Resources

- [GitHub: Removing sensitive data](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)
- [OWASP: Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Fly.io: Secrets Management](https://fly.io/docs/reference/secrets/)

## 🔍 Regular Audits

Run security checks regularly:

```bash
# Check for hardcoded secrets
./scripts/check-secrets.sh

# Review git history
git log --all --full-history --source -- .env

# Scan with gitleaks (if installed)
gitleaks detect --source . --verbose
```

---

**Remember:** Security is everyone's responsibility! 🛡️
