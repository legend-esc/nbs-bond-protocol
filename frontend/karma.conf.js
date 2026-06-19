module.exports = function (config) {
  config.set({
    basePath: '',
    frameworks: ['jasmine'],
    plugins: [
      require('karma-jasmine'),
      require('karma-chrome-launcher'),
    ],
    client: {
      clearContext: false,
    },
    files: [{ pattern: './src/test.ts', watched: false }],
    preprocessors: {
      '**/*.ts': ['@angular-devkit/build-angular/src/tools/webpack/plugins/karma'],
    },
    mime: { 'text/x-typescript': ['ts', 'tsx'] },
    coverageIstanbulReporter: {
      dir: require('path').join(__dirname, './coverage'),
      reports: ['html', 'lcovonly'],
      fixWebpackSourcePaths: true,
    },
    reporters: ['progress'],
    port: 9876,
    colors: true,
    logLevel: config.LOG_INFO,
    autoWatch: true,
    browsers: ['ChromeHeadless'],
    singleRun: false,
    restartOnFileChange: true,
  });
};
