
function saveQuery(query) {
    window.location.hash = encodeURIComponent(query);
}

function getQuery() {
    return decodeURIComponent(window.location.hash.substring(1));
}

function isValidFilter(filter) {
    return filter === "category" || filter === "title";
}

function parseFilter(filter) {
    let cursor = 0;
    let exclude = false;
    
    if (filter.length > 0 && filter[cursor] === "-") {
        exclude = true;
        cursor += 1;
    }
    
    const colon_pos = filter.indexOf(":", cursor);
    
    if (colon_pos < 0 || colon_pos <= cursor) {
        return null;
    }
    
    const filter_key = filter.substring(cursor, colon_pos);
    
    if (!isValidFilter(filter_key)) {
        return null;
    }
    
    cursor = colon_pos + 1;
    
    if (cursor >= filter.length) {
        return null;
    }
    
    const filter_value = filter.substring(cursor).replace(/"/g, "");
    
    return {
        exclude: exclude,
        key: filter_key,
        value: filter_value.toLowerCase(),
    };
}

function isWhitespace(c) {
    return c === " " || c === "\t";
}

function stripQuotes(s) {
    if (s.at(0) === '"' && s.at(-1) === '"') {
        return s.substring(1, s.length - 1);
    } else {
        return s;
    }
}

function splitQuery(query) {
    let tokens = [];
    let inside_quote = false;
    let start = 0;
    
    while (start < query.length) {
        /* Read token */
        let end = start;
        
        while (end < query.length) {
            if (query[end] === '"') {
                inside_quote = !inside_quote;
            }  else if (!inside_quote && isWhitespace(query[end])) {
                break;
            }
            
            end += 1;
        }
        
        if (end > start) {
            let token = query.substring(start, end);
            tokens.push(stripQuotes(token));
        }
        
        /* Skip whitespaces */
        start = end;
        
        while (start < query.length) {
            if (isWhitespace(query[start])) {
                start += 1;
            } else {
                break;
            }
        }
    }
    
    return tokens;
}

function applyFilters(filters) {
    for (elem of document.getElementsByClassName("post-entry")) {
        let show_elem = true;
        
        for (filter of filters) {
            if (!show_elem) {
                break;
            }
            
            if (filter.key === "title") {
                if (elem.getAttribute("data-title").indexOf(filter.value) >= 0) {
                    show_elem = !filter.exclude;
                } else {
                    show_elem = filter.exclude;
                }
            } else if (filter.key === "category") {
                if (elem.getAttribute("data-categories").indexOf(filter.value) >= 0) {
                    show_elem = !filter.exclude;
                } else {
                    show_elem = filter.exclude;
                }
            }
        }
        
        if (show_elem) {
            elem.style.setProperty("display", "block");
        } else {
            elem.style.setProperty("display", "none");
        }
    }
}

function parseQuery(query) {
    const tokens = splitQuery(query);
    let filters = [];
    
    for (token of tokens) {
        const filter = parseFilter(token);
        
        if (filter === null) {
            filters.push({
                exclude: false,
                key: "title",
                value: token.toLowerCase(),
            });
        } else {
            filters.push(filter);
        }
    }
    
    applyFilters(filters);
}

document.addEventListener("DOMContentLoaded", () => {
    const query = getQuery();
    const bar = document.getElementById("search-text");
    const button = document.getElementById("search-submit");
    
    if (query.length > 0) {
        bar.value = query;
        parseQuery(query);
    }
    
    button.addEventListener("click", () => {
        const query = bar.value;
        saveQuery(query);
    });
    
    window.addEventListener("hashchange", () => {
        const query = getQuery();
        bar.value = query;
        parseQuery(query);
    });
});
