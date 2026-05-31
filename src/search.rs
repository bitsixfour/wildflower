
/* This is so that we can use the Enum Expr
 * and Field so that we can get the ID so then it would 
 * PROBABLY be possible to use search.rs to return the fat
 * thingy of data (the full tracklist info) 
 * */
#[derive(Debug, Clone, PartialEq, Eq)]                                                                                                                                                         
pub enum Expr {                                                                                                                                                                                
    And(Field, Field),                                                                                                                                                                         
    Or(Field, Field),                                                                                                                                                                          
    Def(Field),                                                                                                                                                                                
    Empty,                                                                                                                                                                                     
}                                                                                                                                                                                              
#[derive(Debug, Clone, PartialEq, Eq)]                                                                                                                                                         
pub struct Field {                                                                                                                                                                             
    pub field: String,                                                                                                                                                                         
    pub op: bool,                                                                                                                                                                              
    pub value: String                                                                                                                                                                          
}


impl Expr {
    pub fn create_new(&self, album: &Album) -> bool {                                                                                                                                           
       match self {                                                                                                                                                                         
           Expr::And(a, b) => a.matches(album) && b.matches(album),                                                                                                                         
           Expr::Or(a, b)  => a.matches(album) || b.matches(album),                                                                                                                         
           Expr::Def(f)    => f.matches(album),                                                                                                                                             
           Expr::Empty     => true,                                                                                                                                                         
       }                                                                                                                                                                                    
    }        

}
