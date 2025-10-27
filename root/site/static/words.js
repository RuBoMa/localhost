import * as w from './piscineWords.js';

async function createWordObject(questName) {
    const words = w.words[questName];
    const word = words[Math.floor(Math.random() * words.length)];
    const numLettersToRemove = word.length <= 5 ? 1 : 2;
    let result = word.split('');
    let missingLetters = [];
    
    for (let i = 0; i < numLettersToRemove; i++) {
        let indexToRemove;
        do {
            if (numLettersToRemove === 2){
                if (i === 0) indexToRemove = Math.floor(Math.random() * word.length/2);
                if (i === 1) indexToRemove = Math.floor(Math.random() * word.length/2) + Math.floor(word.length/2);
            } else {
                indexToRemove = Math.floor(Math.random() * word.length);
            }
        } while (result[indexToRemove] === '_');
        
        missingLetters.push(result[indexToRemove]);
        result[indexToRemove] = '_';
    }

    return {
    [result.join('')]: missingLetters
    };
}

async function generateWordList(questName) {
    const wordList = [];
    
    for (let i = 0; i < 6; i++) {
        const wordObject = await createWordObject(questName);
        wordList.push(wordObject);
    }
    
    return wordList;
}

function getRandomLetter() {
    const letters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    const randomIndex = Math.floor(Math.random() * letters.length);
    return letters[randomIndex];
}

async function collectMissingLetters(wordList) {
    const missingLetters = [];

    for (let i = 0; i < 5; i++) missingLetters.push(getRandomLetter());

    wordList.forEach(wordObject => {
        const letters = Object.values(wordObject)[0];
        missingLetters.push(...letters);
    });

    for (let i = 0; i < 5; i++) missingLetters.push(getRandomLetter());
    
    return missingLetters;
}

export async function getWordsAndTargets(questName){
    const wordList = await generateWordList(questName);

    const targetList = await collectMissingLetters(wordList);

    return [wordList, targetList];
}