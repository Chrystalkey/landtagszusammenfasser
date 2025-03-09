import pytest
import aiohttp
import datetime
from unittest.mock import Mock, patch, AsyncMock
from ..document import Document, DocumentMeta

@pytest.fixture
def mock_config():
    config = Mock()
    config.llm_connector = Mock()
    config.llm_connector.generate = AsyncMock()
    return config

@pytest.fixture
def sample_pdf_response():
    # This is a minimal valid PDF with metadata
    return (
        b"%PDF-1.4\n"
        b"%\xe2\xe3\xcf\xd3\n"
        b"1 0 obj\n"
        b"<</Type/Catalog/Pages 2 0 R>>\n"
        b"endobj\n"
        b"2 0 obj\n"
        b"<</Type/Pages/Kids[]/Count 0>>\n"
        b"endobj\n"
        b"3 0 obj\n"
        b"<</Title(Test Document)/Author(Test Author)/CreationDate(D:20240101000000Z)/ModDate(D:20240101000000Z)>>\n"
        b"endobj\n"
        b"xref\n"
        b"0 4\n"
        b"0000000000 65535 f \n"
        b"0000000015 00000 n \n"
        b"0000000061 00000 n \n"
        b"0000000111 00000 n \n"
        b"trailer\n"
        b"<</Size 4/Root 1 0 R/Info 3 0 R>>\n"
        b"startxref\n"
        b"213\n"
        b"%%EOF"
    )

@pytest.mark.asyncio
async def test_document_initialization():
    config = Mock()
    doc = Document(None, "http://example.com/test.pdf", "gesetz", config)
    
    assert doc.url == "http://example.com/test.pdf"
    assert doc.typehint == "gesetz"
    assert isinstance(doc.meta, DocumentMeta)
    assert doc.authoren is None
    assert doc.zusammenfassung is None

@pytest.mark.asyncio
async def test_semantic_extraction(mock_config):
    # Mock LLM response
    mock_config.llm_connector.generate.return_value = \
    """Test Title;Group1,Group2;Person1,Person2;Tag1,Tag2;5;Text1,Text2;This is a summary
Test Title;Group1,Group2;Person1,Person2;Tag1,Tag2;5;Text1,Text2;This is a summary"""
    
    doc = Document(None, "http://example.com/test.pdf", "gesetz", mock_config)
    doc.meta.full_text = ["Sample text for testing"]
    
    await doc.extract_semantics()
    
    assert doc.authoren == ["Group1", "Group2"]
    assert doc.autorpersonen == ["Person1", "Person2"]
    assert doc.schlagworte == ["Tag1", "Tag2"]
    assert doc.trojanergefahr == 5
    assert doc.texte == ["Text1", "Text2"]
    assert doc.zusammenfassung == "This is a summary"

@pytest.mark.asyncio
async def test_document_packaging(mock_config):
    doc = Document(None, "http://example.com/test.pdf", "entwurf", mock_config)
    doc.meta.title = "Test Title"
    doc.meta.hash = "testhash"
    doc.meta.typ = "entwurf"
    doc.meta.last_mod = datetime.datetime.now(datetime.timezone.utc)
    doc.authoren = ["Group1"]
    doc.autorpersonen = ["Person1"]
    doc.schlagworte = ["Tag1"]
    doc.zusammenfassung = "Test summary"
    
    packaged = doc.package()
    
    assert packaged.titel == doc.meta.title
    assert packaged.autoren == doc.authoren
    assert packaged.autorpersonen == doc.autorpersonen
    assert packaged.schlagworte == doc.schlagworte
    assert packaged.hash == doc.meta.hash
    assert packaged.typ == doc.meta.typ
    assert packaged.zusammenfassung == doc.zusammenfassung

@pytest.mark.asyncio
async def test_document_json_serialization(mock_config):
    # Create a document with sample data
    original_doc = Document(None, "http://example.com/test.pdf", "gesetz", mock_config)
    original_doc.meta.title = "Test Title"
    original_doc.meta.hash = "testhash"
    original_doc.meta.last_mod = datetime.datetime.now(datetime.timezone.utc)
    original_doc.meta.full_text = ["Sample text"]
    original_doc.meta.typ = "entwurf"
    original_doc.authoren = ["Group1"]
    original_doc.autorpersonen = ["Person1"]
    original_doc.schlagworte = ["Tag1"]
    original_doc.trojanergefahr = 5
    original_doc.texte = ["Text1"]
    original_doc.zusammenfassung = "Test summary"

    # Serialize to JSON
    json_str = original_doc.to_json()
    with open("testout.json", "w") as f:
        f.write(json_str)
    # Deserialize from JSON
    restored_doc = Document.from_json(json_str)
    
    # Verify all attributes are preserved
    assert restored_doc.url == original_doc.url
    assert restored_doc.typehint == original_doc.typehint
    assert restored_doc.meta.title == original_doc.meta.title
    assert restored_doc.meta.hash == original_doc.meta.hash
    assert restored_doc.meta.last_mod.isoformat() == original_doc.meta.last_mod.isoformat()
    assert restored_doc.meta.full_text == original_doc.meta.full_text
    assert restored_doc.meta.typ == original_doc.meta.typ
    assert restored_doc.authoren == original_doc.authoren
    assert restored_doc.autorpersonen == original_doc.autorpersonen
    assert restored_doc.schlagworte == original_doc.schlagworte
    assert restored_doc.trojanergefahr == original_doc.trojanergefahr
    assert restored_doc.texte == original_doc.texte
    assert restored_doc.zusammenfassung == original_doc.zusammenfassung
