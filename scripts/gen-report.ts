import __ from "https://deno.land/x/dirname@1.1.2/mod.ts";
import { existsSync } from "https://deno.land/std@0.105.0/fs/exists.ts";
const { __dirname } = __(import.meta);
import { resolve } from "https://deno.land/std@0.105.0/path/mod.ts";

const TARGET_DIR = Deno.env.get("CARGO_TARGET_DIR") ??
  resolve(__dirname, "../target");
const CRITERION_DIR = resolve(TARGET_DIR, "criterion");

interface Task {
  task: string;
  crate: string;
  time: number;
  timeStdErr: number;
}

interface EstimateReport {
  mean: {
    /**
     * nano second
     */
    point_estimate: number;
    /**
     * nano second
     */
    standard_error: number;
  };
}

interface CratesIoVersion {
  num: string;
  created_at: string;
  yanked: boolean;
}

interface CratesIoCrate {
  max_version: string;
}

interface CratesIoResponse {
  crate: CratesIoCrate;
  versions: CratesIoVersion[];
}

interface CrateMetadata {
  label: string;
}

async function readCrate(crate: string): Promise<Task[]> {
  const crateResultPath = resolve(CRITERION_DIR, crate);
  if (!existsSync(crateResultPath)) {
    throw new Error(
      "crate not found: " + crate +
        ". Run cargo bench first before generating report.",
    );
  }

  const tasks: Task[] = [];
  for await (const dirEntry of Deno.readDir(crateResultPath)) {
    if (dirEntry.isDirectory) {
      tasks.push(
        await readTask(resolve(crateResultPath, dirEntry.name), crate),
      );
    }
  }

  return tasks;
}

async function readTask(path: string, crate: string): Promise<Task> {
  const taskName: string =
    JSON.parse(await Deno.readTextFile(resolve(path, "new", "benchmark.json")))
      .full_id;

  const estimate: EstimateReport = JSON.parse(
    await Deno.readTextFile(resolve(path, "new", "estimates.json")),
  );

  return {
    crate,
    task: taskName.replace(/^.*?\//, ""),
    time: estimate.mean.point_estimate / 1000 / 1000,
    timeStdErr: estimate.mean.standard_error / 1000 / 1000,
  };
}

const BASE_CRATES = [
  "automerge",
  "loro",
  "diamond-types",
  "y-octo",
  "yrs",
] as const;

const PION_CRATE = "crdt(pion)" as const;
const includePion = existsSync(resolve(CRITERION_DIR, PION_CRATE));
const CRATES: readonly string[] = includePion
  ? [...BASE_CRATES, PION_CRATE]
  : [...BASE_CRATES];

async function readCrateSpecFromCargo(
  crate: string,
): Promise<{ version?: string; rev?: string }> {
  const cargoTomlPath = resolve(__dirname, "../Cargo.toml");
  const manifest = await Deno.readTextFile(cargoTomlPath);
  const escapedCrate = crate.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");

  const directMatch = new RegExp(
    `^\\s*${escapedCrate}\\s*=\\s*"([^"\\n]+)"`,
    "m",
  ).exec(manifest);
  if (directMatch?.[1]) {
    return { version: directMatch[1] };
  }

  const inlineTableMatch = new RegExp(
    `^\\s*${escapedCrate}\\s*=\\s*\\{([^\\n]+)\\}`,
    "m",
  ).exec(manifest);
  if (inlineTableMatch?.[1]) {
    const body = inlineTableMatch[1];
    const version = /version\s*=\s*"([^"\n]+)"/.exec(body)?.[1];
    const rev = /rev\s*=\s*"([^"\n]+)"/.exec(body)?.[1];
    if (version || rev) {
      return { version, rev };
    }
  }

  throw new Error(`could not find spec for crate ${crate} in Cargo.toml`);
}

async function fetchCrateMetadata(crate: string): Promise<CrateMetadata> {
  if (crate === PION_CRATE) {
    const pionToml = await Deno.readTextFile(
      resolve(__dirname, "../pion-bench/pion.toml"),
    );
    const rev = /rev\s*=\s*"([^"\n]+)"/.exec(pionToml)?.[1]?.slice(0, 7);
    return {
      label: rev ? `crdt(pion) v0.2.0 (${rev})` : "crdt(pion) v0.2.0",
    };
  }

  const spec = await readCrateSpecFromCargo(crate);
  if (spec.rev && !spec.version) {
    return { label: `${crate} (${spec.rev.slice(0, 7)})` };
  }

  const version = spec.version!;
  const response = await fetch(`https://crates.io/api/v1/crates/${crate}`);
  if (!response.ok) {
    return { label: `${crate} v${version}` };
  }

  const payload: CratesIoResponse = await response.json();
  const versionRecord = payload.versions.find((entry) => entry.num === version);
  const releaseDate = versionRecord?.created_at?.slice(0, 10);

  return {
    label: releaseDate
      ? `${crate} v${version} (${releaseDate})`
      : `${crate} v${version}`,
  };
}

const [results, metadataEntries] = await Promise.all([
  Promise.all(CRATES.map((crate) => readCrate(crate))),
  Promise.all(
    CRATES.map(async (crate) => [crate, await fetchCrateMetadata(crate)] as const),
  ),
]);

const crateMetadata = Object.fromEntries(metadataEntries) as Record<
  string,
  CrateMetadata
>;

interface Transported {
  [task: string]: {
    [crate: string]: {
      time: number;
      timeStdErr: number;
    };
  };
}

const report: Transported = {};
for (let i = 0; i < CRATES.length; i++) {
  for (const task of results[i]) {
    report[task.task] = report[task.task] || {};
    report[task.task][CRATES[i]] = report[task.task][CRATES[i]] || {};
    report[task.task][CRATES[i]].time = task.time;
    report[task.task][CRATES[i]].timeStdErr = task.timeStdErr;
  }
}

const headerCells = CRATES.map((crate) => crateMetadata[crate].label);

console.log(`| Tasks | ${headerCells.join(" | ")} |`);
console.log(`| :---- | ${CRATES.map(() => ":----").join(" | ")} |`);
const tasks = Object.keys(report);
tasks.sort();
for (const task of tasks) {
  console.log(`| ${task} | ${
    CRATES.map((x) => {
      const taskReport = report[task][x];
      if (!taskReport) {
        return "-";
      }
      const time = taskReport.time;
      const timeStdErr = taskReport.timeStdErr;
      if (time > 1) {
        return `${time.toFixed(2)} ± ${timeStdErr.toFixed(2)} ms`;
      } else {
        return `${(time * 1000).toFixed(2)} ± ${
          (timeStdErr * 1000).toFixed(2)
        } us`;
      }
    }).join(" | ")
  } |`);
}
