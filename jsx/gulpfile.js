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
const { spawn } = require('child_process')
const { Writable } = require('stream')
const { basename, join, extname } = require('path');

var DIST_FOLDER = "dist";
var COMBINED = "urban_notes.jxa"

function build() {
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
        .pipe(gulp.src('./src/_bootstrap.js'))
        .pipe(concat(COMBINED))
        .pipe(sourcemaps.write("."))
        .pipe(gulp.dest(DIST_FOLDER))
    ;
    
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

    js_file = join(DIST_FOLDER, COMBINED);
    // pipe the Browserify stream into the stream we created earlier
    // this starts our gulp pipeline.

    b.bundle().pipe(bundledStream).pipe(oscompile([
        '-l', 'JavaScript',
        '-o', join(DIST_FOLDER, basename(js_file, extname(js_file)) + '.app'),
        '-x',
    ]));
    
    return bundledStream;
}

function oscompile (args) {
    let stderr = ''
    const osacompile = spawn('osacompile', args);
  
    osacompile.stderr.on('data', data => { stderr += data })
  
    const stream = new Writable({
      write: osacompile.stdin.write.bind(osacompile.stdin),
      final: function (callback) {
        osacompile.stdin.end()
        osacompile.on('close', function (code) {
          if (!code) return callback()
          callback(new Error(stderr))
        })
      }
    })
  
    osacompile.on('error', stream.emit.bind(stream, 'error'))
  
    return stream
}

exports.default = build;
