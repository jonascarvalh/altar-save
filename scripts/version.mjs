#!/usr/bin/env node
// Mantém a versão do app igual em todos os arquivos.
//
//   node scripts/version.mjs 0.2.0           define a versão
//   node scripts/version.mjs --check 0.2.0   só confere (usado pelo workflow de release)
//
// O tauri.conf.json lê a versão do package.json, então os arquivos são:
// package.json, package-lock.json, src-tauri/Cargo.toml e src-tauri/Cargo.lock.

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const CRATE = "altar-save";

const args = process.argv.slice(2);
const check = args[0] === "--check";
const version = (check ? args[1] : args[0])?.replace(/^v/, "");

if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("Uso: node scripts/version.mjs [--check] <versão>   (ex.: 0.2.0)");
  process.exit(2);
}

/** Cada arquivo: como ler a versão atual e como trocá-la. */
const files = [
  {
    path: "package.json",
    read: (s) => JSON.parse(s).version,
    write: (s) => s.replace(/("version":\s*")[^"]+(")/, `$1${version}$2`),
  },
  {
    path: "package-lock.json",
    read: (s) => {
      const lock = JSON.parse(s);
      return lock.version === lock.packages?.[""]?.version ? lock.version : "(divergente)";
    },
    // A versão do projeto aparece na raiz e em packages[""]: as duas primeiras ocorrências.
    write: (s) => {
      let n = 0;
      return s.replace(/("version":\s*")[^"]+(")/g, (m, a, b) => (n++ < 2 ? `${a}${version}${b}` : m));
    },
  },
  {
    path: "src-tauri/Cargo.toml",
    read: (s) => s.match(/\[package\][^[]*?\nversion\s*=\s*"([^"]+)"/)?.[1],
    write: (s) => s.replace(/(\[package\][^[]*?\nversion\s*=\s*")[^"]+(")/, `$1${version}$2`),
  },
  {
    path: "src-tauri/Cargo.lock",
    read: (s) => s.match(new RegExp(`name = "${CRATE}"\\nversion = "([^"]+)"`))?.[1],
    write: (s) => s.replace(new RegExp(`(name = "${CRATE}"\\nversion = ")[^"]+(")`), `$1${version}$2`),
  },
];

let mismatches = 0;
for (const file of files) {
  const fullPath = join(root, file.path);
  const content = readFileSync(fullPath, "utf8");
  const current = file.read(content);

  if (check) {
    if (current !== version) {
      console.error(`✗ ${file.path}: ${current ?? "(não encontrada)"} (esperado ${version})`);
      mismatches++;
    } else {
      console.log(`✓ ${file.path}: ${current}`);
    }
    continue;
  }

  const updated = file.write(content);
  if (file.read(updated) !== version) {
    console.error(`✗ não consegui atualizar ${file.path}`);
    process.exit(1);
  }
  writeFileSync(fullPath, updated);
  console.log(`${file.path}: ${current} → ${version}`);
}

if (check && mismatches > 0) {
  console.error(`\nA tag pede ${version}, mas ${mismatches} arquivo(s) estão em outra versão.`);
  console.error(`Rode "npm run version:set -- ${version}", faça commit e crie a tag de novo.`);
  process.exit(1);
}
