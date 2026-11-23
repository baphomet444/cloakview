This is a [Next.js](https://nextjs.org) project bootstrapped with [`create-next-app`](https://nextjs.org/docs/app/api-reference/cli/create-next-app).

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/app/building-your-application/optimizing/fonts) to automatically optimize and load [Geist](https://vercel.com/font), a new font family for Vercel.

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/app/building-your-application/deploying) for more details.


## Download gating (`/api/dl`)

The download button points at `/api/dl`, never at GitHub. Release assets on a private
repository answer 404 to anonymous callers, so once the repo is private a direct link dies;
this route fronts the artifact instead and mints a signed, 15-minute URL.

Environment:

- `DL_SECRET` (required, >= 32 chars) signs the token. Missing => 500, on purpose.
- `R2_ACCOUNT_ID` / `R2_ACCESS_KEY_ID` / `R2_SECRET_ACCESS_KEY` (preferred) sign an R2 GET at
  request time and redirect to it. The bucket stays private, the URL lives 5 minutes, and the
  function only ever returns a redirect — no 4.5 MB response limit, no egress through Vercel.
  `R2_BUCKET` (default `cloakview-downloads`) and `R2_KEY` (default `CloakView-Pro_aarch64.dmg`)
  override the object.

  These keys are an **R2 API token** scoped to that one bucket (Object Read & Write — read alone
  cannot publish a release). The same keys live in `~/.cloakview-r2.env` (mode 600) so
  `scripts/release-macos.sh` can push the DMG with `scripts/r2-put.py`, which signs a SigV4 PUT
  with nothing but the standard library. Rotate with R2 → Manage R2 API Tokens → Roll, then
  re-add the two variables here and redeploy.
- `DL_GITHUB_TOKEN` (fallback) a fine-grained PAT with **Contents: read** on the repo. With it
  the asset is streamed through the function.
- `DL_PRESIGN_URL` (fallback) a single presigned R2/S3 URL. It wins over GitHub, but a fixed URL
  expires, so it is only useful for a manual rescue, not for releases.
- `DL_REPO` / `DL_ASSET` override the default repo and asset name.

With none of them set the route just redirects to `browser_download_url`, which is correct only
while the repo is public. Rate limit is 30/min per IP.

Publishing a release puts the DMG in R2 with `scripts/release-macos.sh` (needs
`CLOUDFLARE_API_TOKEN` + `R2_ACCOUNT_ID` in the environment); the site then serves whatever key
`R2_KEY` names, so the object is overwritten per release rather than accreting paths.
