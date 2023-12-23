"use strict";
Object.defineProperty(exports, "__esModule", { value: true });

const path = require("path");
const fs = require("fs");

function native() {
    const modulePath = path.resolve(module.path, './index.node');
    if (!fs.existsSync(modulePath)) {
        throw new Error(`Fail to find native module in: ${modulePath}`);
    }
    return require(modulePath);
}
const nativeModuleRef = native();

const { generateStatement } = nativeModuleRef;
exports.generateStatement = generateStatement;