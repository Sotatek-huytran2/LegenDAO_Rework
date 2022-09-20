var fs = require("fs");
var _ = require("lodash");

const shuffle = (wordsArray) => {
  var tmp,
    current,
    top = wordsArray.length;
  if (top)
    while (--top) {
      current = Math.floor(Math.random() * (top + 1));
      tmp = wordsArray[current];
      wordsArray[current] = wordsArray[top];
      wordsArray[top] = tmp;
    }
  return wordsArray;
};

const getShuffledArrayOfNumbers = (numOfWords) => {
  for (var shuffledArray = [], i = 0; i < numOfWords; ++i) shuffledArray[i] = i;
  shuffledArray = shuffle(shuffledArray);
  return shuffledArray;
};

const extendMnemonicsSetWithOneCommonWordItems = (
  startIndex,
  numOfWords,
  shuffledArray,
  mnemonicsCoordinatesSet
) => {
  let tripletOneCommon = [];

  for (let i = startIndex; i < numOfWords; i += 3) {
    tripletOneCommon.push(shuffledArray[i]);
    if (tripletOneCommon.length === 3) {
      mnemonicsCoordinatesSet.add([...shuffle(tripletOneCommon)]);
      tripletOneCommon = [];
    }
  }
};

const hasDuplicates = (a) => {
  return _.uniq(a).length !== a.length;
};

const assembleMnemonics = (
  mnemonicsCoordinatesSet,
  wordsArray,
  numOfMnemonicsToGenerate
) => {
  let mnemonicsArray = [];

  //   shuffledMnemonics = shuffle(wordsArray.from(mnemonicsCoordinatesSet));
  //   for (item of shuffledMnemonics) {
  for (item of mnemonicsCoordinatesSet) {
    let mnemonic = `${wordsArray[item[0]].trim()} ${wordsArray[
      item[1]
    ].trim()} ${wordsArray[item[2]].trim()}`;

    mnemonicsArray.push(mnemonic);
    numOfMnemonicsToGenerate--;

    if (numOfMnemonicsToGenerate === 0) break;
  }
  return mnemonicsArray;
};

function generateMnemonicsCoordinatesSet(wordsArray) {
  const numOfWords = wordsArray.length;
  var shuffledArray = getShuffledArrayOfNumbers(numOfWords);

  let mnemonicsCoordinatesSet = new Set();
  for (let i = 0; i < numOfWords - (numOfWords % 3); i += 3) {
    let tripletNoCommon = [
      shuffledArray[i],
      shuffledArray[i + 1],
      shuffledArray[i + 2],
    ];
    mnemonicsCoordinatesSet.add(tripletNoCommon);
  }

  for (let startIndex = 0; startIndex < 3; startIndex++) {
    extendMnemonicsSetWithOneCommonWordItems(
      startIndex,
      numOfWords,
      shuffledArray,
      mnemonicsCoordinatesSet
    );
  }
  return mnemonicsCoordinatesSet;
}

const printMnemonics = (mnemonicsArray) => {
  if (hasDuplicates(mnemonicsArray)) {
    console.log("Duplicated mnemonic detected! Aborting");
    return;
  }

  mnemonicsArray.forEach((mnenomic) => {
    console.log(mnenomic);
  });
};

const mnemonicsGenerator = (numOfMnemonicsToGenerate) => {
  // Read 2048 words from a file taken from:
  // https://github.com/bitcoin/bips/blob/master/bip-0039/english.txt
  let wordsArray = fs
    .readFileSync("./bip39_english.txt", "utf-8")
    .toString()
    .split("\n");

  // Generate a set of coordinates of words to take from 'wordsArray'
  let mnemonicsCoordinatesSet = generateMnemonicsCoordinatesSet(wordsArray);

  // Generate the final array of 3-words mnemonics
  mnemonicsArray = assembleMnemonics(
    mnemonicsCoordinatesSet,
    wordsArray,
    numOfMnemonicsToGenerate
  );

  // Print the mnemonics to the console
  printMnemonics(mnemonicsArray);
};

// Generate 'numOfMnemonicsToGenerate' mnemonics composed of 3 words each, with a maximum of 1
// common word between any mnemmonic
const numOfMnemonicsToGenerate = 1000;
mnemonicsGenerator(numOfMnemonicsToGenerate);
