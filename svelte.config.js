import adapter from '@sveltejs/adapter-static';

const config = {
  kit: {
    paths: {
      relative: true
    },
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html'
    })
  }
};

export default config;
