function submitToDb() 
{
    var xmlHttp = new XMLHttpRequest();
    xmlHttp.open( "GET", "http://localhost:8000/session/db", false ); // false for synchronous request
    xmlHttp.send( null );
    //return xmlHttp.responseText;
}
