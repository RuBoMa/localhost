import { shoot } from './shoot.js';
import { game, 
    START, AT_START, 
    INIT, RUNNING, RESTART,
    PAUSED, AT_PAUSED,
    GAME_OVER, COMPLETE, NEXT_LEVEL
} from "./main.js";
import { playSFX } from './audio.js';

export const keysPressed = {
    ArrowUp: false,
    ArrowDown: false,
    ArrowLeft: false,
    ArrowRight: false,
    w: false,
    a: false,
    s: false,
    d: false,
};

const PAUSE_BTN_TEXT = document.getElementsByClassName('pauseText');
const PAUSE_BTN_RECT = document.getElementsByClassName('pauseButton');

export function handleKeyDown(event) {
    if (game.state === AT_START) {
        if (event.key === ' ') game.state = INIT;

    } else if (game.state === RUNNING) {
        handleGameKeys(event);

    } else if (game.state === AT_PAUSED) {
        handlePauseKeys(event);

    } else if (game.state === GAME_OVER) {
        game.difficulty = 0;
        if (event.key === ' ') game.state = RESTART;
    } else if (game.state === COMPLETE) {
        game.difficulty++;
        if (event.key === ' ') game.state = NEXT_LEVEL;
    }

}

export function handleKeyUp(event) {
    if (event.key in keysPressed) {
        keysPressed[event.key] = false;
    }
}

function handleGameKeys(event) {
    if (event.key === ' ') shoot();
    if (event.key === 'Escape') game.state = PAUSED;
    if (event.key in keysPressed) keysPressed[event.key] = true;
}

function handlePauseKeys(event) {
    if (event.key === 'Escape') game.state = RUNNING;
    const prevSelection = game.pauseSelection;
    if (event.key === 'ArrowUp') {
        if (!keysPressed['ArrowUp']) {
            game.pauseSelection = Math.max(0, game.pauseSelection - 1 + 3) % 3;
            playSFX('MENU');
            playSFX('UP');
            updateSelection(prevSelection);
        }
    } else if (event.key === 'ArrowDown') {
        if (!keysPressed['ArrowDown']) {
            game.pauseSelection = (game.pauseSelection + 1) % 3; 
            playSFX('MENU');
            playSFX('UP');
            updateSelection(prevSelection);
        }
    } else if (event.key === ' ') {
        event.preventDefault();
        if (game.pauseSelection === 0) game.state = RUNNING; 
        if (game.pauseSelection === 1) game.state = RESTART; 
        if (game.pauseSelection === 2) {
            game.state = START; 
            const startMenuMusic  = document.getElementById('start-menu-music');
            startMenuMusic.currentTime = 0;
            startMenuMusic.pause();
        } 
        playSFX('SELECT');
    }
}

function updateSelection(prevSelection) {
    PAUSE_BTN_TEXT[prevSelection].setAttribute("fill", "grey");
    PAUSE_BTN_RECT[prevSelection].setAttribute("stroke", "grey");

    PAUSE_BTN_TEXT[game.pauseSelection].setAttribute("fill", "green");
    PAUSE_BTN_RECT[game.pauseSelection].setAttribute("stroke", "green");
}
