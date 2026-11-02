import {runScenario} from '../../tests/native/harness.mjs';
import {checkYoutubePip} from '../../tests/native/pip-youtube.mjs';

await runScenario('media-pip-youtube',checkYoutubePip);
