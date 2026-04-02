import json
import re
import pickle
import os
from pathlib import Path
from typing import List, Dict, Tuple, Set, Optional
from collections import defaultdict
from dataclasses import dataclass, field
import numpy as np
import string
import time

# For advanced search features (optional)
try:
    from rapidfuzz import fuzz, process
    RAPIDFUZZ_AVAILABLE = True
except ImportError:
    RAPIDFUZZ_AVAILABLE = False
    print("Install rapidfuzz for better fuzzy search: pip install rapidfuzz")

try:
    import redis
    REDIS_AVAILABLE = True
except ImportError:
    REDIS_AVAILABLE = False

@dataclass
class Verse:
    """Represents a single verse in the scripture"""
    book: str
    chapter: int
    verse: int
    text: str
    normalized_text: str = ""
    
    def __post_init__(self):
        if not self.normalized_text:
            self.normalized_text = self.normalize_text(self.text)
    
    @staticmethod
    def normalize_text(text: str) -> str:
        """Normalize text for indexing (remove punctuation, lowercase, etc.)"""
        # Convert to lowercase
        text = text.lower()
        # Remove punctuation
        text = text.translate(str.maketrans('', '', string.punctuation))
        # Remove extra whitespace
        text = ' '.join(text.split())
        return text
    
    @property
    def reference(self) -> str:
        return f"{self.book} {self.chapter}:{self.verse}"
    
    def __str__(self):
        return f"{self.reference}: {self.text}"

@dataclass
class SearchResult:
    """Search result with relevance scoring"""
    verse: Verse
    score: float
    matches: List[str] = field(default_factory=list)
    highlighted_text: str = ""
    
    def __lt__(self, other):
        return self.score > other.score  # For heapq (higher score first)

class InvertedIndex:
    """High-performance inverted index for scripture search"""
    
    def __init__(self):
        self.index: Dict[str, Dict[int, List[int]]] = defaultdict(lambda: defaultdict(list))
        self.verses: List[Verse] = []
        self.word_positions: Dict[int, Dict[str, List[int]]] = defaultdict(dict)
        self.total_verses = 0
        self.stop_words = self._load_stop_words()
        
    def _load_stop_words(self) -> Set[str]:
        """Load common stop words"""
        return {
            'a', 'an', 'and', 'the', 'of', 'to', 'in', 'for', 'on', 'with',
            'by', 'at', 'from', 'is', 'was', 'were', 'are', 'be', 'been',
            'being', 'have', 'has', 'had', 'having', 'do', 'does', 'did',
            'doing', 'but', 'or', 'so', 'for', 'nor', 'yet', 'as', 'into',
            'through', 'during', 'before', 'after', 'above', 'below', 'between'
        }
    
    def add_verse(self, verse: Verse):
        """Add a verse to the index"""
        verse_id = len(self.verses)
        self.verses.append(verse)
        
        # Tokenize and index words with positions
        words = verse.normalized_text.split()
        for position, word in enumerate(words):
            if word not in self.stop_words and len(word) > 1:
                # Store position for phrase search
                self.index[word][verse_id].append(position)
                self.word_positions[verse_id][word] = [position]
        
        self.total_verses = len(self.verses)
    
    def search_term(self, term: str) -> Dict[int, List[int]]:
        """Search for a single term"""
        term = term.lower().strip()
        return self.index.get(term, {})
    
    def search_phrase(self, phrase: str) -> List[Tuple[int, int]]:
        """Search for exact phrase using positional indexes"""
        words = phrase.lower().split()
        if not words:
            return []
        
        # Get verses containing first word
        first_word_results = self.index.get(words[0], {})
        
        matches = []
        for verse_id, positions in first_word_results.items():
            # Check if subsequent words exist at correct positions
            for pos in positions:
                match = True
                for i, word in enumerate(words[1:]):
                    next_pos = pos + i + 1
                    if next_pos not in self.index.get(word, {}).get(verse_id, []):
                        match = False
                        break
                if match:
                    matches.append((verse_id, pos))
        
        return matches
    
    def boolean_search(self, query: str) -> Set[int]:
        """Boolean search with AND, OR, NOT operators"""
        # Parse simple boolean query
        query = query.lower()
        
        # Handle AND
        if ' and ' in query:
            terms = query.split(' and ')
            result_sets = [set(self.index.get(term.strip(), {}).keys()) for term in terms]
            return set.intersection(*result_sets) if result_sets else set()
        
        # Handle OR
        elif ' or ' in query:
            terms = query.split(' or ')
            result_sets = [set(self.index.get(term.strip(), {}).keys()) for term in terms]
            return set.union(*result_sets)
        
        # Handle NOT
        elif ' not ' in query:
            parts = query.split(' not ')
            include_terms = parts[0].split(' and ') if ' and ' in parts[0] else [parts[0]]
            exclude_terms = parts[1].split(' and ') if ' and ' in parts[1] else [parts[1]]
            
            include_set = set.intersection(*[set(self.index.get(t.strip(), {}).keys()) for t in include_terms])
            exclude_set = set.union(*[set(self.index.get(t.strip(), {}).keys()) for t in exclude_terms])
            
            return include_set - exclude_set
        
        # Single term
        else:
            return set(self.index.get(query, {}).keys())

class TFIDFScorer:
    """TF-IDF scoring for relevance ranking"""
    
    def __init__(self, index: InvertedIndex):
        self.index = index
        self.idf_cache = {}
        
    def compute_idf(self, term: str) -> float:
        """Compute IDF for a term"""
        if term in self.idf_cache:
            return self.idf_cache[term]
        
        doc_count = len(self.index.index.get(term, {}))
        if doc_count == 0:
            return 0
        
        idf = np.log(self.index.total_verses / (1 + doc_count))
        self.idf_cache[term] = idf
        return idf
    
    def compute_tf(self, term: str, verse_id: int, positions: List[int]) -> float:
        """Compute TF for a term in a verse"""
        # Use log normalization
        freq = len(positions)
        if freq == 0:
            return 0
        return 1 + np.log(freq)
    
    def score_verse(self, verse_id: int, query_terms: List[str]) -> float:
        """Compute TF-IDF score for a verse given query terms"""
        score = 0.0
        verse = self.index.verses[verse_id]
        
        for term in query_terms:
            if term in self.index.index:
                positions = self.index.index[term].get(verse_id, [])
                if positions:
                    tf = self.compute_tf(term, verse_id, positions)
                    idf = self.compute_idf(term)
                    score += tf * idf
        
        return score

class ScriptureSearchEngine:
    """High-performance scripture search engine"""
    
    def __init__(self, use_cache=True, cache_size=1000):
        self.index = InvertedIndex()
        self.scorer = None
        self.cache = {}
        self.cache_size = cache_size
        self.use_cache = use_cache
        self.search_history = []
        
    def load_scripture(self, data_source):
        """Load scripture from various sources"""
        if isinstance(data_source, str):
            if data_source.endswith('.json'):
                self._load_from_json(data_source)
            elif data_source.endswith('.txt'):
                self._load_from_text(data_source)
            else:
                raise ValueError("Unsupported file format")
        elif isinstance(data_source, list):
            self._load_from_list(data_source)
        else:
            raise ValueError("Data source must be file path or list")
        
        self.scorer = TFIDFScorer(self.index)
        
    def _load_from_json(self, json_file: str):
        """Load from JSON format"""
        with open(json_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        # Support different JSON structures
        if isinstance(data, list):
            for item in data:
                verse = Verse(
                    book=item.get('book', ''),
                    chapter=item.get('chapter', 0),
                    verse=item.get('verse', 0),
                    text=item.get('text', '')
                )
                self.index.add_verse(verse)
        elif isinstance(data, dict):
            for book, chapters in data.items():
                for chapter, verses in chapters.items():
                    for verse_num, text in verses.items():
                        verse = Verse(
                            book=book,
                            chapter=int(chapter),
                            verse=int(verse_num),
                            text=text
                        )
                        self.index.add_verse(verse)
    
    def _load_from_text(self, text_file: str):
        """Load from plain text with format: Book Chapter:Verse Text"""
        with open(text_file, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                
                # Parse format: "Genesis 1:1 In the beginning..."
                match = re.match(r'^([A-Za-z\s]+)\s+(\d+):(\d+)\s+(.+)$', line)
                if match:
                    verse = Verse(
                        book=match.group(1).strip(),
                        chapter=int(match.group(2)),
                        verse=int(match.group(3)),
                        text=match.group(4)
                    )
                    self.index.add_verse(verse)
    
    def _load_from_list(self, verses_list):
        """Load from list of Verse objects or dictionaries"""
        for item in verses_list:
            if isinstance(item, Verse):
                self.index.add_verse(item)
            elif isinstance(item, dict):
                verse = Verse(
                    book=item.get('book', ''),
                    chapter=item.get('chapter', 0),
                    verse=item.get('verse', 0),
                    text=item.get('text', '')
                )
                self.index.add_verse(verse)
    
    def search(self, query: str, limit: int = 50, search_type: str = 'auto',
               highlight: bool = True) -> List[SearchResult]:
        """Main search interface"""
        
        # Check cache
        cache_key = f"{query}:{limit}:{search_type}"
        if self.use_cache and cache_key in self.cache:
            return self.cache[cache_key]
        
        # Determine search type
        if search_type == 'auto':
            if ' and ' in query or ' or ' in query or ' not ' in query:
                search_type = 'boolean'
            elif ' ' in query and len(query.split()) > 1:
                search_type = 'phrase'
            else:
                search_type = 'term'
        
        # Execute search
        start_time = time.time()
        
        if search_type == 'phrase':
            results = self._phrase_search(query, limit)
        elif search_type == 'boolean':
            results = self._boolean_search(query, limit)
        elif search_type == 'fuzzy':
            results = self._fuzzy_search(query, limit)
        else:
            results = self._term_search(query, limit)
        
        # Highlight text if requested
        if highlight:
            for result in results:
                result.highlighted_text = self._highlight_text(
                    result.verse.text, 
                    result.matches
                )
        
        # Cache results
        if self.use_cache:
            if len(self.cache) >= self.cache_size:
                # Remove oldest entry
                self.cache.pop(next(iter(self.cache)))
            self.cache[cache_key] = results
        
        # Record search history
        self.search_history.append({
            'query': query,
            'type': search_type,
            'results_count': len(results),
            'time': time.time() - start_time
        })
        
        return results
    
    def _term_search(self, query: str, limit: int) -> List[SearchResult]:
        """Simple term search with TF-IDF scoring"""
        query_terms = query.lower().split()
        query_terms = [t for t in query_terms if t not in self.index.stop_words]
        
        # Collect candidate verses
        candidate_verses = set()
        for term in query_terms:
            candidate_verses.update(self.index.index.get(term, {}).keys())
        
        # Score verses
        scored_results = []
        for verse_id in candidate_verses:
            score = self.scorer.score_verse(verse_id, query_terms)
            verse = self.index.verses[verse_id]
            
            # Collect matching terms
            matches = [term for term in query_terms 
                      if term in self.index.index and verse_id in self.index.index[term]]
            
            scored_results.append((score, verse, matches))
        
        # Sort by score and limit
        scored_results.sort(key=lambda x: x[0], reverse=True)
        
        return [SearchResult(verse=verse, score=score, matches=matches) 
                for score, verse, matches in scored_results[:limit]]
    
    def _phrase_search(self, query: str, limit: int) -> List[SearchResult]:
        """Exact phrase search"""
        matches = self.index.search_phrase(query)
        
        results = []
        for verse_id, position in matches[:limit]:
            verse = self.index.verses[verse_id]
            results.append(SearchResult(
                verse=verse,
                score=1.0,
                matches=[query]
            ))
        
        return results
    
    def _boolean_search(self, query: str, limit: int) -> List[SearchResult]:
        """Boolean search with AND/OR/NOT operators"""
        verse_ids = self.index.boolean_search(query)
        
        results = []
        for verse_id in list(verse_ids)[:limit]:
            verse = self.index.verses[verse_id]
            results.append(SearchResult(
                verse=verse,
                score=1.0,
                matches=[query]
            ))
        
        return results
    
    def _fuzzy_search(self, query: str, limit: int, threshold: int = 80) -> List[SearchResult]:
        """Fuzzy search using rapidfuzz"""
        if not RAPIDFUZZ_AVAILABLE:
            raise ImportError("Install rapidfuzz for fuzzy search: pip install rapidfuzz")
        
        # Collect all unique words from index
        all_words = list(self.index.index.keys())
        
        # Find similar words
        similar_words = process.extract(query, all_words, scorer=fuzz.WRatio, limit=20)
        similar_words = [word for word, score in similar_words if score >= threshold]
        
        # Search using similar words
        candidate_verses = set()
        for word in similar_words:
            candidate_verses.update(self.index.index.get(word, {}).keys())
        
        # Score results
        results = []
        for verse_id in candidate_verses:
            verse = self.index.verses[verse_id]
            # Simple scoring based on number of matches
            score = sum(1 for word in similar_words 
                       if word in self.index.index and verse_id in self.index.index[word])
            results.append(SearchResult(verse=verse, score=score, matches=similar_words[:5]))
        
        results.sort(key=lambda x: x.score, reverse=True)
        return results[:limit]
    
    def _highlight_text(self, text: str, matches: List[str], 
                       highlight_start: str = "**", highlight_end: str = "**") -> str:
        """Highlight matching terms in text"""
        if not matches:
            return text
        
        highlighted = text
        for match in matches:
            # Case-insensitive replacement
            pattern = re.compile(re.escape(match), re.IGNORECASE)
            highlighted = pattern.sub(f"{highlight_start}\\g<0>{highlight_end}", highlighted)
        
        return highlighted
    
    def search_by_reference(self, reference: str) -> Optional[Verse]:
        """Search by book chapter:verse reference"""
        # Parse reference (e.g., "Genesis 1:1" or "John 3:16")
        match = re.match(r'^([A-Za-z\s]+)\s+(\d+):(\d+)$', reference)
        if match:
            book = match.group(1).strip()
            chapter = int(match.group(2))
            verse_num = int(match.group(3))
            
            for verse in self.index.verses:
                if (verse.book == book and 
                    verse.chapter == chapter and 
                    verse.verse == verse_num):
                    return verse
        
        return None
    
    def search_range(self, book: str, start_chapter: int, start_verse: int,
                    end_chapter: int, end_verse: int) -> List[Verse]:
        """Search a range of verses"""
        results = []
        for verse in self.index.verses:
            if verse.book == book:
                if (verse.chapter > start_chapter or 
                    (verse.chapter == start_chapter and verse.verse >= start_verse)):
                    if (verse.chapter < end_chapter or 
                        (verse.chapter == end_chapter and verse.verse <= end_verse)):
                        results.append(verse)
        return results
    
    def get_statistics(self) -> Dict:
        """Get search engine statistics"""
        return {
            'total_verses': self.index.total_verses,
            'unique_words': len(self.index.index),
            'total_word_occurrences': sum(len(positions) for word in self.index.index.values() 
                                         for positions in word.values()),
            'cache_size': len(self.cache),
            'search_history_count': len(self.search_history),
            'fuzzy_available': RAPIDFUZZ_AVAILABLE,
            'redis_available': REDIS_AVAILABLE
        }
    
    def save_index(self, filepath: str):
        """Save index to disk for fast loading"""
        with open(filepath, 'wb') as f:
            pickle.dump({
                'index': dict(self.index.index),
                'verses': self.index.verses,
                'total_verses': self.index.total_verses
            }, f)
    
    def load_index(self, filepath: str):
        """Load pre-built index from disk"""
        with open(filepath, 'rb') as f:
            data = pickle.load(f)
        
        self.index.index = defaultdict(lambda: defaultdict(list), data['index'])
        self.index.verses = data['verses']
        self.index.total_verses = data['total_verses']
        self.scorer = TFIDFScorer(self.index)
