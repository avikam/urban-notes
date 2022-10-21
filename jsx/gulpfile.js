var browserify = require('browserify');
var babelify = require("babelify");

var gulp = require('gulp');
var source = require('vinyl-source-stream');
var buffer = require('vinyl-buffer');
var through = require('through2');
var sourcemaps = require("gulp-sourcemaps");
var babel = require("gulp-babel");
var concat = require("gulp-concat");
var log = require('gulplog');


gulp.task("default", function () {
    var bundledStream = through();

    bundledStream
        // turns the output bundle stream into a stream containing
        // the normal attributes gulp plugins expect.
        .pipe(source('app.js')) // filename is a "pretend" filename to use for your file
        .pipe(buffer())
        // the rest of the gulp task, as you would normally write it.
        .pipe(sourcemaps.init())
        .pipe(babel({
            presets: ['@babel/preset-env']
        }))
        .pipe(concat("urban_notes.jxa"))
        .pipe(sourcemaps.write("."))
        .pipe(gulp.dest("dist"));
    
    // Browserify creates it's own readable stream.
    const entries = [
        './src/main.js',
        // './src/parse.js',
    ];
    
    // create the Browserify instance.
    var b = browserify({
        entries: entries,
        debug: true
    }).transform(babelify);

    // pipe the Browserify stream into the stream we created earlier
    // this starts our gulp pipeline.
    b.bundle().pipe(bundledStream);
    
    return bundledStream;
});