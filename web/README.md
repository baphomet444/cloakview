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
- `DL_GITHUB_TOKEN` (set once the repo is private) a fine-grained PAT with **Contents: read**
  on the repo. With it the asset is streamed through the function.
- `DL_PRESIGN_URL` (optional) a presigned R2/S3 URL; when set it wins over GitHub and the
  storage host is never exposed.
- `DL_REPO` / `DL_ASSET` override the default repo and asset name.

With none of the token modes set the route just redirects to `browser_download_url`, which
is correct only while the repo is public. Rate limit is 30/min per IP.
