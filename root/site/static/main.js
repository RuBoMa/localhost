import { initPlayer, player, updatePlayerPosition } from './player.js';
import { moveTargets, initTargets } from './target.js';
import { initUI } from './ui.js';
import { handleKeyDown, handleKeyUp } from './input.js';
import { hideAllMenus, showMenu } from './showMenu.js'
import { initBlock, moveBlock } from './block.js';
import { getWordsAndTargets } from './words.js';
import { playMusic, resetAudioFlags, initAutoplayPolicy } from './audio.js'

// Constants for game states
export const START = 'start';
export const AT_START = 'atStart';
export const INIT = 'initalize';
export const RUNNING = 'running';
export const PAUSED = 'paused';
export const AT_PAUSED = 'atPaused';
export const GAME_OVER = 'gameover';
export const COMPLETE = 'complete';
export const RESTART = 'restart';
export const NEXT_LEVEL = 'nextLevel';

export const game = {
    height: 600,
    width: 800,
    currentQuest: 'questOne',
    state: START,
    timeDuration: 60,
    wordList: [],
    currentWordIndex: 0,
    targetList: [],
    targets: [],
    currentTargetIndex: 0,
    pauseSelection: 0,
    difficulty: 0
};

//--------------- start gameLoop ---------------//
export function main() {
    initAutoplayPolicy();
    gameLoop();
}

//--------------- game logic ---------------//
function gameLoop(timestamp) {
    if (game.state === START) {
        playMusic();
        hideAllMenus();
        showMenu();

    } else if (game.state === INIT || game.state === RESTART || game.state === NEXT_LEVEL){
        const points = (game.state === NEXT_LEVEL) ? player.score : 0;
        initGame(points);

    } else if (game.state === RUNNING){
        hideAllMenus();
        updatePlayerPosition();
        moveTargets();
        moveBlock();
        playMusic();

    } else if (game.state === PAUSED) {
        playMusic();
        showMenu();

    }else if (game.state === GAME_OVER) {
        playMusic();
        showMenu();

    }else if (game.state === COMPLETE) {
        playMusic();
        showMenu();
    }
    renderFps(timestamp);
    requestAnimationFrame(gameLoop);
}

// initalize game 
async function initGame(points) {
    resetAudioFlags();

    [game.wordList, game.targetList] = await getWordsAndTargets(game.currentQuest);
    initPlayer(points);
    initTargets();
    initBlock();
    initUI();

    game.state = RUNNING;
}

//--------------- FPS counter ---------------//
const fpsDisplay = document.getElementById("fps-display");

let frameCount = 0;
let lastFPSTime = 0;
let currentFPS;

async function renderFps(timestamp) {
    frameCount++;

    const elapsedSinceLastFPS = timestamp - lastFPSTime;
    if (elapsedSinceLastFPS >= 1000) {
        currentFPS = frameCount;
        frameCount = 0;
        lastFPSTime = timestamp;
    }

    fpsDisplay.textContent = `FPS: ${currentFPS}`;
}

//--------------- Add eventListener for user input ---------------//
document.addEventListener('keydown', handleKeyDown);
document.addEventListener('keyup', handleKeyUp);
