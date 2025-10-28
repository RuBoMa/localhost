import { game, START, RUNNING, PAUSED, GAME_OVER, COMPLETE } from "./main.js";

const bgm = document.getElementById('start-menu-music');
const music = document.getElementById('start-menu-music');
const gameOverMusic = document.getElementById('game-over-music');
const successMusic = document.getElementById('success-music');

// -- In-Game Music --- //

let audioFlags = {
    gameOver: false,
    success: false,
};

export function playMusic(){
    if (game.state === START) playBGM();
    if (game.state === PAUSED) playPauseMenuMusic();
    if (game.state === GAME_OVER) playGameOverMusic();
    if (game.state === COMPLETE) playSuccessMusic();
    if (game.state === RUNNING) playBGM();
}

export function playBGM() {
    bgm.volume = 0.8
    const playPromise = bgm.play();

    if (playPromise !== undefined) {
    playPromise.then(_ => {
    })
    .catch(error => {
        console.log('Autoplay failed: ', error);
    });
    }
}

function playPauseMenuMusic() {
    music.volume = 0.1;
    music.play();
}

function playGameOverMusic() {
    music.pause();

    if (!audioFlags.gameOver) {
        gameOverMusic.play();    
        audioFlags.gameOver = true;       
    }
}

function playSuccessMusic() {
    music.pause();
    
    if (!audioFlags.success) {
        successMusic.play();
        audioFlags.success = true;  
    }
}

export function resetAudioFlags() {
    for (const flag in audioFlags) {
        audioFlags[flag] = false;
    }
}

// -- Player UI Feedback --- //

export function playSFX(type) {
    const soundMap = {
        MISS: document.getElementById('miss-sound'),
        HIT: document.getElementById('bim-sound'),
        BIM: document.getElementById('bim-sound'),
        MENU: document.getElementById('menu-sound'),
        UP: document. getElementById('up-sound'),
        SELECT: document.getElementById('select-sound'),
    };

    if (soundMap[type]) {
        soundMap[type].play();
    } else {
        console.error('Undefined Sound Type');
    }
}

// -- Chrome's Autoplay Policy -- //

let bgmPlayed = false;

export function initAutoplayPolicy() {
    ['keydown', 'click'].forEach(eventType => {
        document.addEventListener(eventType, function allowPlay() {
            if (!bgmPlayed) {
                playBGM();
                bgmPlayed = true;
            }
            document.removeEventListener(eventType, allowPlay);
        }, { once: true });
    });
}
