# frozen_string_literal: true
require "bundler/setup"

def crlfify(file_path)
  content = File.read(file_path)
  crlf_content = content.gsub(/\r?\n/, "\r\n")
  File.write(file_path, crlf_content)
end

desc "ビルドします"
task :build do
  sh "cargo build --release"
end

desc "リリースアセットを作成します"
task :release => [:build] do
  require "tomlrb"

  version = Tomlrb.load_file("./Cargo.toml")["package"]["version"]
  rm_rf "release" if Dir.exist?("release")
  mkdir "release"

  release_md = File.read("./release.md")
  File.write("./release/README.md", release_md.gsub("{{version}}", version))

  sh "cargo about generate ./about.package.hbs -o ./release/package.txt"
  sh "cargo about generate ./about.hbs -o ./release/THIRD_PARTY_NOTICES.md"
  sh "au2 release --set-version #{version}"
end
