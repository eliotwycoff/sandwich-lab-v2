// This helper function applies the passed callback function 
// to the DOM element given by `elementId` after it's been rendered.
const setElement = (elementId, callback) => {
    const element = document.getElementById(elementId);

    if (element) {
        callback(element);
    } else {
        window.requestAnimationFrame(() => {
            setElement(elementId, callback);
        });
    }
};

const fetch_pair = (pairAddress, apiEndpoint) => {
    // under construction
};