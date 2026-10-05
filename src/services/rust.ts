import type { User } from "@bindings/User";
import { callRust, dataDir } from "rust-bridge";

export class SystemService {
  static init() {
    return callRust({
      cmd: "system",
      args: { type: "init", data_dir: dataDir() },
    });
  }

  static ping() {
    return callRust({ cmd: "system", args: { type: "ping" } });
  }

  static version() {
    return callRust({ cmd: "system", args: { type: "get_version" } });
  }
}

export class MathService {
  static factorial(n: number) {
    return callRust({ cmd: "math", args: { type: "factorial", n } });
  }
}

export class UserService {
  static get(id: string) {
    return callRust({ cmd: "user", args: { type: "get_user", id } });
  }

  static save(user: User) {
    return callRust({ cmd: "user", args: { type: "save_user", ...user } });
  }
}
