int main(void) {
  int a; 

  for(a = 0; a < 5; a = a+1) {
    if (a == 2) {
      break; 
    }
  }

  return a;
}
